# Security

The plugin ships with a strict default posture. Every choice described here is the behavior with no configuration; the few knobs that exist are scoped, one-shot, or explicitly named.

## Transport

TLS is provided by rustls, statically linked. There is no fallback to native TLS, no opportunistic downgrade, no support for the operating system's certificate store override — the WebPKI roots compiled into the binary are the only trust anchors. HTTP/2 is enabled. The system proxy environment variables are explicitly disabled at client construction time, so the plugin never honors `HTTP_PROXY`, `HTTPS_PROXY`, or `NO_PROXY`.

Two timeouts apply to every outgoing call:

- A **connect timeout** of seven seconds bounds the TCP and TLS handshake combined.
- A **request timeout** of twelve seconds bounds the whole exchange, including reading the response body.

These values are not configurable from Pawn. They are tight enough that a slow remote service will not stall the worker pool indefinitely, and generous enough that a healthy WAN call is unaffected.

## URL validation

The submitted URL is parsed before the request is dispatched. Anything that is not a valid `http://` or `https://` URL is rejected with `HTTPS_ERROR_BAD_URL` before any network activity occurs. The scheme is checked explicitly; URLs with `file://`, `data:`, or any other scheme are rejected even if syntactically well-formed.

## Redirect policy

Automatic redirects are disabled at the HTTP client layer and re-implemented manually with explicit rules. Up to five redirects are followed; the sixth is refused with `HTTPS_ERROR_POLICY_BLOCKED`.

### Anti-downgrade

If any URL in the redirect chain used `https://`, no subsequent step is allowed to use `http://`. The first step that would downgrade aborts the request with `HTTPS_ERROR_POLICY_BLOCKED`. This applies even when the initial URL is plain HTTP and an intermediate redirect bounces through HTTPS — once secure, always secure.

### Cross-host policy

A redirect to a different host is refused unless the script explicitly opted in by calling `https_allow_cross_host_once(true)` immediately before submitting the request. The flag is one-shot: it is consumed at submission time and reset, so each call requires a fresh opt-in. Without it, a redirect that changes the host fails with `HTTPS_ERROR_POLICY_BLOCKED`.

When cross-host is allowed and a redirect actually crosses hosts, the `Authorization` header is stripped from the follow-up request. This is unconditional and case-insensitive; credentials are never forwarded silently to a different origin.

### Method semantics across redirects

The follow-up request method is chosen according to the same rules most HTTP clients apply:

- `303 See Other` always becomes a `GET` and discards the body.
- `301 Moved Permanently` and `302 Found` turn a `POST` into a `GET` and discard the body; other methods are preserved.
- `307 Temporary Redirect` and `308 Permanent Redirect` preserve both the method and the body.

### Malformed redirects

A `3xx` response without a `Location` header, or with a `Location` value that does not resolve against the current URL, is refused with `HTTPS_ERROR_POLICY_BLOCKED`. The redirect target is resolved relative to the current URL, so both absolute and relative `Location` values are supported.

### HEAD does not redirect

HEAD requests never auto-follow. The 3xx status is reported back to Pawn verbatim, the body is empty as usual, and the error code is zero (the request itself succeeded, the server just answered with a redirection). If the script needs to follow that redirect, it can resubmit explicitly using the `Location` it learned out-of-band.

## Resource caps

- The response body size is capped per request to `https_get_max_body_bytes`, default 64 KiB, range 4 KiB to 1 MiB inclusive. Exceeding the cap returns `HTTPS_ERROR_CONTENT_TOO_BIG`. The cap is applied to the decompressed bytes.
- The pending response queue holds at most 1024 entries. Overflow drops the oldest entry, never the newest — older entries are likely to belong to scripts that are no longer interested.
- The PEM file loader refuses files larger than 256 KiB. A real client identity is well under that ceiling; the limit exists to make file-system mistakes loud rather than silent.

## Mutual TLS

A client identity can be installed from a combined PEM buffer with `https_mtls_set_pem` or from a file on disk with `https_mtls_set_pem_file`. Both natives expect the certificate and the private key in the same PEM blob; that is the standard format for client identities and is what rustls accepts.

Once installed, the identity is applied to every subsequent request from a dedicated HTTP client that mirrors the default client's settings (same timeouts, same anti-proxy stance, same rustls configuration). The default client remains in use for requests issued before the identity was installed.

`https_mtls_clear` removes the identity. After the call, subsequent requests use the default client again.

mTLS is global, not per-request. There is no facility to swap identities between calls — install the one you need, dispatch the requests that need it, and clear when done. If the application needs multiple identities concurrently, that is currently out of scope.

## What the plugin does not do

The plugin does not pin certificates. It does not retry failed requests. It does not deduplicate concurrent identical calls. It does not enforce any content-type matching on the response. It does not validate the JSON of the response (validation is only on the request side, opt-in via `https_jsonf`). It does not buffer responses on disk; everything happens in memory and is bounded by the caps described above. None of these are likely to change without an explicit feature request.
