use crate::state;
use crossbeam_channel::{Sender, TrySendError, bounded};
use reqwest::blocking::{Client, RequestBuilder, multipart};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::Read;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use url::Url;

const MAX_REDIRECTS: usize = 5;
const JOB_QUEUE_BOUND: usize = 256;
const MIN_WORKERS: usize = 2;
const MAX_WORKERS: usize = 8;

/// Error codes mirrored in `https_samp.inc`.
#[repr(i32)]
#[derive(Clone, Copy)]
enum ErrorCode {
    None = 0,
    BadUrl = 1,
    TlsHandshake = 2,
    NoSocket = 3,
    CantConnect = 4,
    SendFail = 5,
    ContentTooBig = 6,
    Timeout = 7,
    PolicyBlocked = 8,
    Unknown = 10,
}

impl ErrorCode {
    fn as_i32(self) -> i32 {
        self as i32
    }
}

pub enum BodyPayload {
    None,
    Raw(String),
    Multipart(state::MultipartPayload),
}

pub struct Job {
    pub index: i32,
    pub method: String,
    pub url: String,
    pub body: BodyPayload,
    pub callback: String,
    pub headers: Vec<(String, String)>,
    pub allow_cross_host: bool,
    pub total_timeout: Option<Duration>,
}

static JOB_TX: LazyLock<Sender<Job>> = LazyLock::new(|| {
    let (tx, rx) = bounded::<Job>(JOB_QUEUE_BOUND);
    let workers = num_cpus::get().clamp(MIN_WORKERS, MAX_WORKERS);
    for _ in 0..workers {
        let rx = rx.clone();
        thread::spawn(move || {
            for job in rx.iter() {
                run_job(job);
            }
        });
    }
    tx
});

/// Submits a request. Falls back to a one-off thread if the worker pool is saturated.
pub fn start_request(index: i32, method: String, url: String, body: BodyPayload, callback: String) {
    let headers: Vec<(String, String)> = state::snapshot_headers().into_iter().collect();
    let job = Job {
        index,
        method,
        url,
        body,
        callback,
        headers,
        allow_cross_host: state::take_allow_cross_host_once(),
        total_timeout: state::take_timeout_once(),
    };

    if let Err(err) = JOB_TX.try_send(job) {
        let recovered = match err {
            TrySendError::Full(j) | TrySendError::Disconnected(j) => j,
        };
        thread::spawn(move || run_job(recovered));
    }
}

fn finish_with_error(index: i32, callback: &str, status: i32, error: ErrorCode) {
    state::enqueue_response(
        index,
        callback.to_string(),
        String::new(),
        status,
        error.as_i32(),
        HashMap::new(),
    );
    state::clear_temp_headers();
}

fn finish_ok(
    index: i32,
    callback: &str,
    body: String,
    status: i32,
    headers: HashMap<String, String>,
) {
    state::enqueue_response(
        index,
        callback.to_string(),
        body,
        status,
        ErrorCode::None.as_i32(),
        headers,
    );
    state::clear_temp_headers();
}

fn build_multipart_form(payload: state::MultipartPayload) -> Option<multipart::Form> {
    let mut form = multipart::Form::new();
    for (k, v) in payload.text {
        form = form.text(k, v);
    }
    for (field, filename, path) in payload.files {
        let part = multipart::Part::file(&path).ok()?.file_name(filename);
        form = form.part(field, part);
    }
    Some(form)
}

fn attach_body(req: RequestBuilder, body: BodyPayload) -> Option<RequestBuilder> {
    match body {
        BodyPayload::None => Some(req),
        BodyPayload::Raw(s) => Some(req.body(s)),
        BodyPayload::Multipart(parts) => {
            let form = build_multipart_form(parts)?;
            Some(req.multipart(form))
        }
    }
}

fn run_job(job: Job) {
    let Job {
        index,
        mut method,
        url,
        body,
        callback,
        mut headers,
        allow_cross_host,
        total_timeout,
    } = job;

    let mut current = match Url::parse(&url) {
        Ok(u) if u.scheme() == "http" || u.scheme() == "https" => u,
        _ => {
            finish_with_error(index, &callback, 0, ErrorCode::BadUrl);
            return;
        }
    };

    let client: Arc<Client> = state::active_client();
    let mut redirects = 0usize;
    let mut saw_https_ever = current.scheme() == "https";
    // The body is consumed once; subsequent redirects either drop it (303,
    // 301/302 from POST) or preserve it as already-sent. To preserve across
    // 307/308 we keep an owned copy.
    let mut body = Some(body);

    loop {
        let mut req = match method.as_str() {
            "POST" => client.post(current.clone()),
            "PUT" => client.put(current.clone()),
            "PATCH" => client.patch(current.clone()),
            "DELETE" => client.delete(current.clone()),
            "HEAD" => client.head(current.clone()),
            _ => client.get(current.clone()),
        };

        if let Some(t) = total_timeout {
            req = req.timeout(t);
        }

        for (k, v) in headers.iter() {
            req = req.header(k, v);
        }

        let body_for_this = body.take().unwrap_or(BodyPayload::None);
        // Multipart cannot be replayed across redirects; if we need to preserve
        // a body for 307/308, downgrade Multipart to None on the second attempt.
        // For Raw bodies we clone before sending so the original survives.
        let (body_to_send, body_to_keep) = match body_for_this {
            BodyPayload::None => (BodyPayload::None, BodyPayload::None),
            BodyPayload::Raw(s) => (BodyPayload::Raw(s.clone()), BodyPayload::Raw(s)),
            BodyPayload::Multipart(m) => (BodyPayload::Multipart(m), BodyPayload::None),
        };
        body = Some(body_to_keep);

        let Some(req_with_body) = attach_body(req, body_to_send) else {
            finish_with_error(index, &callback, 0, ErrorCode::Unknown);
            return;
        };

        let resp = match req_with_body.send() {
            Ok(r) => r,
            Err(e) => {
                finish_with_error(index, &callback, 0, map_reqwest_error(&e));
                return;
            }
        };

        let status = resp.status();

        if method == "HEAD" {
            let resp_headers = extract_response_headers(&resp);
            finish_ok(
                index,
                &callback,
                String::new(),
                status.as_u16() as i32,
                resp_headers,
            );
            return;
        }

        if status.is_redirection() {
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|hv| hv.to_str().ok())
                .unwrap_or("");

            if location.is_empty() || redirects >= MAX_REDIRECTS {
                finish_with_error(index, &callback, 0, ErrorCode::PolicyBlocked);
                return;
            }

            let next_url = match current.join(location) {
                Ok(u) => u,
                Err(_) => {
                    finish_with_error(index, &callback, 0, ErrorCode::PolicyBlocked);
                    return;
                }
            };

            if saw_https_ever && next_url.scheme() == "http" {
                finish_with_error(index, &callback, 0, ErrorCode::PolicyBlocked);
                return;
            }
            if next_url.scheme() == "https" {
                saw_https_ever = true;
            }

            let host_changed = next_url.host_str() != current.host_str();
            if host_changed && !allow_cross_host {
                finish_with_error(index, &callback, 0, ErrorCode::PolicyBlocked);
                return;
            }

            if host_changed {
                headers.retain(|(k, _)| !k.eq_ignore_ascii_case("authorization"));
            }

            match status.as_u16() {
                303 => {
                    method = "GET".into();
                    body = Some(BodyPayload::None);
                }
                301 | 302 if method == "POST" => {
                    method = "GET".into();
                    body = Some(BodyPayload::None);
                }
                _ => {}
            }

            current = next_url;
            redirects += 1;
            continue;
        }

        let resp_headers = extract_response_headers(&resp);
        let limit = state::max_body_bytes();
        let mut reader = resp.take((limit + 1) as u64);
        let mut buf = Vec::with_capacity(limit.min(16 * 1024));
        match reader.read_to_end(&mut buf) {
            Ok(_) if buf.len() > limit => {
                finish_with_error(
                    index,
                    &callback,
                    status.as_u16() as i32,
                    ErrorCode::ContentTooBig,
                );
            }
            Ok(_) => {
                let text = String::from_utf8_lossy(&buf).into_owned();
                finish_ok(index, &callback, text, status.as_u16() as i32, resp_headers);
            }
            Err(e) => {
                finish_with_error(index, &callback, 0, map_io_error(&e));
            }
        }
        return;
    }
}

fn extract_response_headers(resp: &reqwest::blocking::Response) -> HashMap<String, String> {
    let mut out = HashMap::with_capacity(resp.headers().len());
    for (name, value) in resp.headers().iter() {
        if let Ok(v) = value.to_str() {
            out.insert(name.as_str().to_ascii_lowercase(), v.to_string());
        }
    }
    out
}

fn map_reqwest_error(e: &reqwest::Error) -> ErrorCode {
    if e.is_timeout() {
        return ErrorCode::Timeout;
    }
    if e.is_request() || e.is_builder() {
        return ErrorCode::BadUrl;
    }
    if let Some(ioe) = find_io_error(e) {
        return map_io_error(ioe);
    }
    if e.is_connect() {
        return ErrorCode::TlsHandshake;
    }
    if e.is_status() {
        return ErrorCode::None;
    }
    ErrorCode::Unknown
}

fn map_io_error(e: &std::io::Error) -> ErrorCode {
    use std::io::ErrorKind::*;
    match e.kind() {
        TimedOut => ErrorCode::Timeout,
        BrokenPipe | WriteZero => ErrorCode::SendFail,
        ConnectionRefused => ErrorCode::CantConnect,
        NotConnected | AddrNotAvailable | NetworkUnreachable | HostUnreachable => {
            ErrorCode::NoSocket
        }
        _ => ErrorCode::Unknown,
    }
}

fn find_io_error(e: &reqwest::Error) -> Option<&std::io::Error> {
    let mut cur: Option<&(dyn StdError + 'static)> = e.source();
    while let Some(err) = cur {
        if let Some(ioe) = err.downcast_ref::<std::io::Error>() {
            return Some(ioe);
        }
        cur = err.source();
    }
    None
}
