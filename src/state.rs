use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use reqwest::redirect::Policy;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use url::form_urlencoded;

const MAX_QUEUE: usize = 1024;
const MIN_BODY_BYTES: usize = 4 * 1024;
const MAX_BODY_BYTES_CAP: usize = 1024 * 1024;
const DEFAULT_BODY_BYTES: usize = 64 * 1024;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(7);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(12);
const USER_AGENT: &str = "SA-MP HTTPS/1.0";

static MAX_BODY_BYTES: AtomicUsize = AtomicUsize::new(DEFAULT_BODY_BYTES);
static ALLOW_CROSS_HOST_ONCE: AtomicBool = AtomicBool::new(false);
static PENDING_TIMEOUT_MS: AtomicU64 = AtomicU64::new(0);
static COOKIES_ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct HttpsResponse {
    pub index: i32,
    pub callback: String,
    pub response: String,
    pub status: i32,
    pub error: i32,
    pub headers: HashMap<String, String>,
}

// ============================================================================
// Request headers
// ============================================================================

static GLOBAL_HEADERS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static TEMP_HEADERS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set_global_header(key: String, value: String) {
    if let Ok(mut g) = GLOBAL_HEADERS.lock() {
        g.insert(key, value);
    }
}

pub fn clear_global_headers() {
    if let Ok(mut g) = GLOBAL_HEADERS.lock() {
        g.clear();
    }
}

pub fn set_temp_header(key: String, value: String) {
    if let Ok(mut t) = TEMP_HEADERS.lock() {
        t.insert(key, value);
    }
}

pub fn clear_temp_headers() {
    if let Ok(mut t) = TEMP_HEADERS.lock() {
        t.clear();
    }
}

pub fn snapshot_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), USER_AGENT.to_string());
    if let Ok(g) = GLOBAL_HEADERS.lock() {
        headers.extend(g.clone());
    }
    if let Ok(t) = TEMP_HEADERS.lock() {
        headers.extend(t.clone());
    }
    headers
}

// ============================================================================
// Response queue
// ============================================================================

static RESPONSE_QUEUE: LazyLock<Mutex<VecDeque<HttpsResponse>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub fn enqueue_response(
    index: i32,
    callback: String,
    response: String,
    status: i32,
    error: i32,
    headers: HashMap<String, String>,
) {
    if take_cancelled(index) {
        return;
    }
    if let Ok(mut q) = RESPONSE_QUEUE.lock() {
        if q.len() >= MAX_QUEUE {
            q.pop_front();
        }
        q.push_back(HttpsResponse { index, callback, response, status, error, headers });
    }
}

pub fn drain_responses(limit: usize) -> Vec<HttpsResponse> {
    let mut out = Vec::new();
    if let Ok(mut q) = RESPONSE_QUEUE.lock() {
        let take = limit.min(q.len());
        for _ in 0..take {
            if let Some(item) = q.pop_front() {
                out.push(item);
            }
        }
    }
    out
}

pub fn queue_len() -> usize {
    RESPONSE_QUEUE.lock().map(|q| q.len()).unwrap_or(0)
}

// ============================================================================
// Body-size limit
// ============================================================================

pub fn set_max_body_bytes(bytes: usize) -> usize {
    let clamped = bytes.clamp(MIN_BODY_BYTES, MAX_BODY_BYTES_CAP);
    MAX_BODY_BYTES.store(clamped, Ordering::Relaxed);
    clamped
}

pub fn max_body_bytes() -> usize {
    MAX_BODY_BYTES.load(Ordering::Relaxed)
}

// ============================================================================
// Cross-host redirect (one-shot flag)
// ============================================================================

pub fn set_allow_cross_host_once(enable: bool) {
    ALLOW_CROSS_HOST_ONCE.store(enable, Ordering::Relaxed);
}

pub fn take_allow_cross_host_once() -> bool {
    ALLOW_CROSS_HOST_ONCE.swap(false, Ordering::AcqRel)
}

// ============================================================================
// Per-request total timeout (one-shot, in milliseconds; 0 = use default)
// ============================================================================

pub fn set_timeout_once(total_ms: u64) {
    PENDING_TIMEOUT_MS.store(total_ms, Ordering::Relaxed);
}

pub fn take_timeout_once() -> Option<Duration> {
    let ms = PENDING_TIMEOUT_MS.swap(0, Ordering::AcqRel);
    if ms == 0 { None } else { Some(Duration::from_millis(ms)) }
}

// ============================================================================
// Cancellation set: response delivery is skipped for cancelled indices
// ============================================================================

static CANCELLED: LazyLock<Mutex<HashSet<i32>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn cancel(index: i32) {
    if let Ok(mut c) = CANCELLED.lock() {
        c.insert(index);
    }
}

pub fn take_cancelled(index: i32) -> bool {
    if let Ok(mut c) = CANCELLED.lock() {
        c.remove(&index)
    } else {
        false
    }
}

// ============================================================================
// Pending POST/PUT/PATCH payload (one-shot, priority: multipart > raw > json > form)
// ============================================================================

#[derive(Default)]
pub struct MultipartPayload {
    pub text: Vec<(String, String)>,
    pub files: Vec<(String, String, String)>,
}

impl MultipartPayload {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.files.is_empty()
    }
}

#[derive(Default)]
struct PendingPayload {
    raw: Option<String>,
    json: Option<String>,
    form: Vec<(String, String)>,
    multipart: MultipartPayload,
}

static PENDING_PAYLOAD: LazyLock<Mutex<PendingPayload>> =
    LazyLock::new(|| Mutex::new(PendingPayload::default()));

fn clear_other_payloads(p: &mut PendingPayload) {
    p.raw = None;
    p.json = None;
    p.form.clear();
    p.multipart.text.clear();
    p.multipart.files.clear();
}

pub fn set_body_raw(s: String) -> bool {
    if s.len() > max_body_bytes() {
        return false;
    }
    if let Ok(mut p) = PENDING_PAYLOAD.lock() {
        clear_other_payloads(&mut p);
        p.raw = Some(s);
        return true;
    }
    false
}

pub fn set_body_json(s: String) -> bool {
    if s.len() > max_body_bytes() {
        return false;
    }
    if serde_json::from_str::<serde_json::Value>(&s).is_err() {
        return false;
    }
    if let Ok(mut p) = PENDING_PAYLOAD.lock() {
        clear_other_payloads(&mut p);
        p.json = Some(s);
        return true;
    }
    false
}

pub fn add_form_pair(key: String, value: String) -> bool {
    let Ok(mut p) = PENDING_PAYLOAD.lock() else {
        return false;
    };

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (k, v) in p.form.iter() {
        serializer.append_pair(k, v);
    }
    serializer.append_pair(&key, &value);
    if serializer.finish().len() > max_body_bytes() {
        return false;
    }

    p.raw = None;
    p.json = None;
    p.multipart.text.clear();
    p.multipart.files.clear();
    p.form.push((key, value));
    true
}

pub fn add_multipart_text(key: String, value: String) -> bool {
    let Ok(mut p) = PENDING_PAYLOAD.lock() else {
        return false;
    };
    p.raw = None;
    p.json = None;
    p.form.clear();
    p.multipart.text.push((key, value));
    true
}

pub fn add_multipart_file(field: String, filename: String, path: String) -> bool {
    if std::fs::metadata(&path).is_err() {
        return false;
    }
    let Ok(mut p) = PENDING_PAYLOAD.lock() else {
        return false;
    };
    p.raw = None;
    p.json = None;
    p.form.clear();
    p.multipart.files.push((field, filename, path));
    true
}

pub enum PreparedBody {
    None,
    Raw(String, Option<String>),
    Multipart(MultipartPayload),
}

pub fn take_prepared_body() -> PreparedBody {
    let Ok(mut p) = PENDING_PAYLOAD.lock() else {
        return PreparedBody::None;
    };

    if !p.multipart.is_empty() {
        let mut m = MultipartPayload::default();
        std::mem::swap(&mut m, &mut p.multipart);
        return PreparedBody::Multipart(m);
    }
    if let Some(s) = p.raw.take() {
        return PreparedBody::Raw(s, None);
    }
    if let Some(s) = p.json.take() {
        return PreparedBody::Raw(s, Some("application/json; charset=utf-8".to_string()));
    }
    if !p.form.is_empty() {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (k, v) in p.form.drain(..) {
            serializer.append_pair(&k, &v);
        }
        return PreparedBody::Raw(
            serializer.finish(),
            Some("application/x-www-form-urlencoded".to_string()),
        );
    }
    PreparedBody::None
}

// ============================================================================
// Cookies
// ============================================================================

static COOKIE_JAR: LazyLock<Mutex<Arc<Jar>>> =
    LazyLock::new(|| Mutex::new(Arc::new(Jar::default())));

fn cookie_jar() -> Arc<Jar> {
    COOKIE_JAR.lock().map(|j| j.clone()).unwrap_or_else(|_| Arc::new(Jar::default()))
}

pub fn set_cookies_enabled(enabled: bool) {
    COOKIES_ENABLED.store(enabled, Ordering::Relaxed);
    rebuild_active_client();
}

/// Replaces the cookie jar with a fresh, empty one and rebuilds the active
/// client so subsequent requests do not see any previously stored cookies.
pub fn clear_cookies() {
    if let Ok(mut jar) = COOKIE_JAR.lock() {
        *jar = Arc::new(Jar::default());
    }
    rebuild_active_client();
}

// ============================================================================
// Active client (mTLS + cookies aware)
// ============================================================================

static MTLS_PEM: LazyLock<Mutex<Option<Vec<u8>>>> = LazyLock::new(|| Mutex::new(None));
static ACTIVE_CLIENT: LazyLock<Mutex<Arc<Client>>> =
    LazyLock::new(|| Mutex::new(Arc::new(build_client(None, false).expect("default client"))));

fn build_client(mtls_pem: Option<&[u8]>, cookies: bool) -> Result<Client, reqwest::Error> {
    let mut b = Client::builder()
        .use_rustls_tls()
        .no_proxy()
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .timeout(DEFAULT_REQUEST_TIMEOUT)
        .redirect(Policy::none());

    if cookies {
        b = b.cookie_provider(cookie_jar());
    }
    if let Some(pem) = mtls_pem
        && let Ok(identity) = reqwest::Identity::from_pem(pem)
    {
        b = b.identity(identity);
    }
    b.build()
}

fn rebuild_active_client() {
    let pem_bytes: Option<Vec<u8>> = MTLS_PEM.lock().ok().and_then(|g| g.clone());
    let cookies = COOKIES_ENABLED.load(Ordering::Relaxed);

    if let Ok(client) = build_client(pem_bytes.as_deref(), cookies)
        && let Ok(mut active) = ACTIVE_CLIENT.lock()
    {
        *active = Arc::new(client);
    }
}

pub fn active_client() -> Arc<Client> {
    ACTIVE_CLIENT.lock().map(|c| c.clone()).unwrap_or_else(|_| {
        Arc::new(build_client(None, false).expect("fallback default client"))
    })
}

pub fn set_mtls_identity_pem(pem: &[u8]) -> bool {
    if reqwest::Identity::from_pem(pem).is_err() {
        return false;
    }
    if let Ok(mut g) = MTLS_PEM.lock() {
        *g = Some(pem.to_vec());
    }
    rebuild_active_client();
    true
}

pub fn clear_mtls_identity() {
    if let Ok(mut g) = MTLS_PEM.lock() {
        *g = None;
    }
    rebuild_active_client();
}
