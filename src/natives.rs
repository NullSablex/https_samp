use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use samp::cell::AmxString;
use samp::native;
use samp::prelude::*;

use crate::https::{self, BodyPayload};
use crate::state::{self, PreparedBody};
use crate::{Plugin, util};

const POST: i32 = 2;
const HEAD: i32 = 3;
const PUT: i32 = 4;
const DELETE: i32 = 5;
const PATCH: i32 = 6;
const PEM_FILE_MAX_BYTES: u64 = 256 * 1024;

fn method_name(request_type: i32) -> &'static str {
    match request_type {
        POST => "POST",
        HEAD => "HEAD",
        PUT => "PUT",
        DELETE => "DELETE",
        PATCH => "PATCH",
        _ => "GET",
    }
}

fn method_accepts_body(request_type: i32) -> bool {
    matches!(request_type, POST | PUT | PATCH)
}

impl Plugin {
    /// `https(index, type, url, data, callback)` — submits an HTTPS request.
    #[native(name = "https")]
    pub fn https(
        &mut self,
        _amx: &Amx,
        index: i32,
        request_type: i32,
        url: &AmxString,
        data: &AmxString,
        callback: &AmxString,
    ) -> AmxResult<bool> {
        let method = method_name(request_type).to_string();

        let body = if method_accepts_body(request_type) {
            let inline = data.to_string();
            if !inline.is_empty() {
                BodyPayload::Raw(inline)
            } else {
                match state::take_prepared_body() {
                    PreparedBody::None => BodyPayload::None,
                    PreparedBody::Raw(s, ct) => {
                        if let Some(ct) = ct {
                            state::set_temp_header("Content-Type".to_string(), ct);
                        }
                        BodyPayload::Raw(s)
                    }
                    PreparedBody::Multipart(parts) => BodyPayload::Multipart(parts),
                }
            }
        } else {
            BodyPayload::None
        };

        https::start_request(index, method, url.to_string(), body, callback.to_string());
        Ok(true)
    }

    #[native(name = "https_set_header")]
    pub fn https_set_header(
        &mut self,
        _amx: &Amx,
        key: &AmxString,
        value: &AmxString,
    ) -> AmxResult<bool> {
        state::set_temp_header(key.to_string(), value.to_string());
        Ok(true)
    }

    #[native(name = "https_set_global_header")]
    pub fn https_set_global_header(
        &mut self,
        _amx: &Amx,
        key: &AmxString,
        value: &AmxString,
    ) -> AmxResult<bool> {
        state::set_global_header(key.to_string(), value.to_string());
        Ok(true)
    }

    #[native(name = "https_clear_global_headers")]
    pub fn https_clear_global_headers(&mut self, _amx: &Amx) -> AmxResult<bool> {
        state::clear_global_headers();
        Ok(true)
    }

    /// Reads a response header (case-insensitive) of the response currently
    /// being delivered to the calling Pawn public. Returns false outside of
    /// a callback or when the header is absent.
    #[native(name = "https_response_header")]
    pub fn https_response_header(
        &mut self,
        _amx: &Amx,
        key: &AmxString,
        dest: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<bool> {
        let key_str = key.to_string();
        let Some(value) = util::current_header(&key_str) else {
            dest.write_str(size, "")?;
            return Ok(false);
        };
        dest.write_str(size, &value)?;
        Ok(true)
    }

    /// Backwards-compatibility no-op: since rust-samp v3 the unified `on_tick`
    /// already drains the response queue automatically.
    #[native(name = "https_process_queue")]
    pub fn https_process_queue(&mut self, _amx: &Amx) -> AmxResult<bool> {
        Ok(true)
    }

    #[native(name = "https_set_max_body_bytes")]
    pub fn https_set_max_body_bytes(&mut self, _amx: &Amx, bytes: i32) -> AmxResult<i32> {
        let applied = state::set_max_body_bytes(bytes.max(0) as usize);
        Ok(applied as i32)
    }

    #[native(name = "https_get_max_body_bytes")]
    pub fn https_get_max_body_bytes(&mut self, _amx: &Amx) -> AmxResult<i32> {
        Ok(state::max_body_bytes() as i32)
    }

    #[native(name = "https_queue_len")]
    pub fn https_queue_len(&mut self, _amx: &Amx) -> AmxResult<i32> {
        Ok(state::queue_len() as i32)
    }

    #[native(name = "https_allow_cross_host_once")]
    pub fn https_allow_cross_host_once(&mut self, _amx: &Amx, enable: bool) -> AmxResult<bool> {
        state::set_allow_cross_host_once(enable);
        Ok(true)
    }

    /// Sets a per-request total timeout in milliseconds (one-shot, consumed at
    /// submission). Pass 0 to revert to the default.
    #[native(name = "https_set_timeout_once")]
    pub fn https_set_timeout_once(&mut self, _amx: &Amx, total_ms: i32) -> AmxResult<bool> {
        state::set_timeout_once(total_ms.max(0) as u64);
        Ok(true)
    }

    /// Marks an index as cancelled. If a response for that index is still in
    /// the dispatch queue when this is called, it will be dropped. Already
    /// delivered callbacks are not affected.
    #[native(name = "https_cancel")]
    pub fn https_cancel(&mut self, _amx: &Amx, index: i32) -> AmxResult<bool> {
        state::cancel(index);
        Ok(true)
    }

    // -------- Body builders --------

    #[native(name = "https_bodyf")]
    pub fn https_bodyf(&mut self, _amx: &Amx, data: &AmxString) -> AmxResult<bool> {
        Ok(state::set_body_raw(data.to_string()))
    }

    #[native(name = "https_jsonf")]
    pub fn https_jsonf(&mut self, _amx: &Amx, data: &AmxString) -> AmxResult<bool> {
        if !state::set_body_json(data.to_string()) {
            return Ok(false);
        }
        state::set_temp_header(
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        Ok(true)
    }

    #[native(name = "https_form_add")]
    pub fn https_form_add(
        &mut self,
        _amx: &Amx,
        key: &AmxString,
        value: &AmxString,
    ) -> AmxResult<bool> {
        if !state::add_form_pair(key.to_string(), value.to_string()) {
            return Ok(false);
        }
        state::set_temp_header(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        );
        Ok(true)
    }

    /// Appends a text field to the multipart/form-data builder.
    #[native(name = "https_multipart_add_text")]
    pub fn https_multipart_add_text(
        &mut self,
        _amx: &Amx,
        key: &AmxString,
        value: &AmxString,
    ) -> AmxResult<bool> {
        Ok(state::add_multipart_text(key.to_string(), value.to_string()))
    }

    /// Appends a file field to the multipart/form-data builder. The file is
    /// opened by the worker thread at send time; this native only validates
    /// that the path exists.
    #[native(name = "https_multipart_add_file")]
    pub fn https_multipart_add_file(
        &mut self,
        _amx: &Amx,
        field: &AmxString,
        filename: &AmxString,
        path: &AmxString,
    ) -> AmxResult<bool> {
        Ok(state::add_multipart_file(
            field.to_string(),
            filename.to_string(),
            path.to_string(),
        ))
    }

    // -------- Authentication helpers --------

    #[native(name = "https_set_basic_auth_once")]
    pub fn https_set_basic_auth_once(
        &mut self,
        _amx: &Amx,
        user: &AmxString,
        password: &AmxString,
    ) -> AmxResult<bool> {
        let token = BASE64.encode(format!("{}:{}", user, password));
        state::set_temp_header("Authorization".to_string(), format!("Basic {}", token));
        Ok(true)
    }

    #[native(name = "https_set_bearer_once")]
    pub fn https_set_bearer_once(&mut self, _amx: &Amx, token: &AmxString) -> AmxResult<bool> {
        state::set_temp_header("Authorization".to_string(), format!("Bearer {}", token));
        Ok(true)
    }

    // -------- Cookies --------

    #[native(name = "https_cookies_enable")]
    pub fn https_cookies_enable(&mut self, _amx: &Amx, enable: bool) -> AmxResult<bool> {
        state::set_cookies_enabled(enable);
        Ok(true)
    }

    #[native(name = "https_cookies_clear")]
    pub fn https_cookies_clear(&mut self, _amx: &Amx) -> AmxResult<bool> {
        state::clear_cookies();
        Ok(true)
    }

    // -------- mTLS --------

    #[native(name = "https_mtls_set_pem")]
    pub fn https_mtls_set_pem(&mut self, _amx: &Amx, pem: &AmxString) -> AmxResult<bool> {
        Ok(state::set_mtls_identity_pem(&pem.to_bytes()))
    }

    #[native(name = "https_mtls_set_pem_file")]
    pub fn https_mtls_set_pem_file(&mut self, _amx: &Amx, path: &AmxString) -> AmxResult<bool> {
        Ok(load_pem_file(&path.to_string())
            .map(|buf| state::set_mtls_identity_pem(&buf))
            .unwrap_or(false))
    }

    #[native(name = "https_mtls_clear")]
    pub fn https_mtls_clear(&mut self, _amx: &Amx) -> AmxResult<bool> {
        state::clear_mtls_identity();
        Ok(true)
    }
}

fn load_pem_file(path: &str) -> Option<Vec<u8>> {
    use std::fs;
    use std::io::Read;

    let meta = fs::metadata(path).ok()?;
    if meta.len() > PEM_FILE_MAX_BYTES {
        return None;
    }

    let mut buf = Vec::with_capacity(meta.len() as usize);
    fs::File::open(path).ok()?.read_to_end(&mut buf).ok()?;
    Some(buf)
}
