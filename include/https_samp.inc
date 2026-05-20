/*
 * https_samp v1.0.0 - HTTPS client plugin for SA-MP and Open Multiplayer.
 * Author:     NullSablex
 * Repository: https://github.com/NullSablex/https_samp
 * License:    AGPL-3.0-or-later
 */

#if defined _https_samp_included
    #endinput
#endif
#define _https_samp_included

#define HTTPS_SAMP_VERSION "1.0.0"

#pragma library https

// Request types
enum {
    HTTPS_GET = 1,
    HTTPS_POST,
    HTTPS_HEAD,
    HTTPS_PUT,
    HTTPS_DELETE,
    HTTPS_PATCH,
}

// Error codes
enum {
    HTTPS_ERROR_NONE = 0,
    HTTPS_ERROR_BAD_URL,
    HTTPS_ERROR_TLS_HANDSHAKE,
    HTTPS_ERROR_NO_SOCKET,
    HTTPS_ERROR_CANT_CONNECT,
    HTTPS_ERROR_SEND_FAIL,
    HTTPS_ERROR_CONTENT_TOO_BIG,
    HTTPS_ERROR_TIMEOUT,
    HTTPS_ERROR_POLICY_BLOCKED,
    HTTPS_ERROR_UNKNOWN = 10,
}

// Request
native https(index, type, const url[], const data[], const callback[]);

// Cancellation
native bool:https_cancel(index);

// Request headers
native bool:https_set_header(const key[], const value[]);
native bool:https_set_global_header(const key[], const value[]);
native bool:https_clear_global_headers();

// Response headers (call inside the request callback)
native bool:https_response_header(const key[], dest[], max_len = sizeof(dest));

// Body builders (one-shot, consumed by the next POST/PUT/PATCH)
native bool:https_bodyf(const data[]);
native bool:https_jsonf(const data[]);
native bool:https_form_add(const key[], const value[]);
native bool:https_multipart_add_text(const key[], const value[]);
native bool:https_multipart_add_file(const field[], const filename[], const path[]);

// Authentication helpers (one-shot)
native bool:https_set_basic_auth_once(const user[], const password[]);
native bool:https_set_bearer_once(const token[]);

// Queue
// Kept for backwards compatibility only: since rust-samp v3.0.0 the unified
// on_tick already dispatches callbacks automatically on SA-MP and native open.mp.
native https_process_queue();
native https_queue_len();

// Limits
native https_set_max_body_bytes(bytes);
native https_get_max_body_bytes();

// Timeouts
native bool:https_set_timeout_once(total_ms);

// Security
native bool:https_allow_cross_host_once(bool:enable);

// Cookies
native bool:https_cookies_enable(bool:enable);
native bool:https_cookies_clear();

// mTLS
native bool:https_mtls_set_pem(const pem[]);
native bool:https_mtls_set_pem_file(const path[]);
native bool:https_mtls_clear();
