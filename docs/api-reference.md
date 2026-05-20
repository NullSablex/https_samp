# API reference

This page lists every native exposed by the plugin, grouped by purpose. Refer to the linked guide pages for the rationale and the broader context.

## Request

### `https(index, type, url, data, callback)`

Submits a request to the worker pool and returns immediately.

- `index` is an arbitrary integer, echoed back to the callback unchanged.
- `type` is one of `HTTPS_GET`, `HTTPS_POST`, `HTTPS_HEAD`, `HTTPS_PUT`, `HTTPS_DELETE`, `HTTPS_PATCH`.
- `url` is the absolute URL. Both `http://` and `https://` are accepted at the entry point; the redirect policy refuses downgrades later.
- `data` is the request body. Used only for `POST`, `PUT`, and `PATCH`. If empty for those, the staged builder payload is consumed instead.
- `callback` is the name of the Pawn `public` to invoke when the response is ready.

Return value: success indicator. The actual outcome is reported via the callback.

### `https_cancel(index)`

Marks an index as cancelled. If the response was not yet enqueued or has not been delivered to its Pawn callback, the result is discarded. The network request itself may still complete in the background — only the callback delivery is suppressed.

### Callback signature

The callback must accept four parameters in this order:

- `index` — the integer supplied to `https`.
- `response[]` — the response body as a string. Empty for HEAD and for every failure path other than `CONTENT_TOO_BIG`.
- `status` — the HTTP status code, or zero when no HTTP response was produced.
- `error` — one of the `HTTPS_ERROR_*` codes documented in [Error codes](error-codes.md).

## Request headers

### `https_set_header(key, value)`

Adds a header to the temporary table. The temporary table is cleared automatically after each request, regardless of outcome.

### `https_set_global_header(key, value)`

Adds a header to the global table. Persists across requests until explicitly cleared. Overridden by a temporary header with the same name for the duration of one request.

### `https_clear_global_headers()`

Empties the global header table. Does not touch the temporary table or the built-in `User-Agent`.

## Response headers

### `https_response_header(key, dest[], max_len)`

Reads a response header (case-insensitive name lookup) of the response currently being delivered to the calling Pawn public. Writes the value to `dest` truncated to `max_len`. Returns `true` when the header is present, `false` otherwise (the buffer is filled with an empty string in that case).

Must be called from inside the request callback; calling it from anywhere else returns `false` with an empty buffer.

## Authentication helpers

### `https_set_basic_auth_once(user, password)`

Stages an `Authorization: Basic <base64(user:password)>` header for the next request, in the temporary header table. Consumed and cleared like any other temporary header.

### `https_set_bearer_once(token)`

Stages an `Authorization: Bearer <token>` header for the next request, in the temporary header table.

## Body builders

### `https_bodyf(data)`

Stages a raw body for the next POST, PUT, or PATCH. Does not set `Content-Type`. Rejects the call if the body exceeds the current size cap.

### `https_jsonf(data)`

Validates `data` as JSON and stages it for the next POST, PUT, or PATCH. On success, also installs `Content-Type: application/json; charset=utf-8` as a temporary header. Rejects invalid JSON.

### `https_form_add(key, value)`

Appends a key/value pair to the form-urlencoded payload accumulator. On success, also installs `Content-Type: application/x-www-form-urlencoded` as a temporary header. Rejects the call if the accumulated body would exceed the size cap.

### `https_multipart_add_text(key, value)`

Appends a text field to the `multipart/form-data` builder.

### `https_multipart_add_file(field, filename, path)`

Appends a file field to the `multipart/form-data` builder. The file at `path` is opened by the worker thread at send time; this native only validates that the path exists.

## Limits and queue

### `https_set_max_body_bytes(bytes)`

Sets the maximum response body size. The argument is clamped to the inclusive range 4 KiB to 1 MiB. Returns the value actually applied after clamping.

### `https_get_max_body_bytes()`

Returns the currently effective response body size cap.

### `https_set_timeout_once(total_ms)`

Overrides the default total request timeout (twelve seconds) for the next call only. Pass `0` to revert to the default. The connect timeout cannot be overridden per request.

### `https_queue_len()`

Returns the number of responses waiting to be dispatched to Pawn.

### `https_process_queue()`

Backwards-compatibility no-op. The unified tick provided by the rust-samp v3 runtime already drains the queue automatically.

## Security

### `https_allow_cross_host_once(enable)`

Sets a one-shot flag permitting the next request to follow redirects across hosts. Consumed at submission time.

## Cookies

### `https_cookies_enable(enable)`

Toggles the cookie store. When enabled, cookies received from a server are persisted in an in-memory jar and re-sent on subsequent requests to the same host. Disabled by default.

### `https_cookies_clear()`

Replaces the cookie jar with a fresh, empty one. Subsequent requests start with no stored cookies.

## mTLS

### `https_mtls_set_pem(pem)`

Installs a mutual TLS identity from a combined PEM buffer containing the client certificate and private key. Returns whether the identity was accepted.

### `https_mtls_set_pem_file(path)`

Same as `https_mtls_set_pem`, but reads the PEM blob from a file on disk. Files larger than 256 KiB are refused.

### `https_mtls_clear()`

Removes the installed mTLS identity. Subsequent requests revert to the default client.

## Constants

The following constants are defined in `https_samp.inc`.

### Request types

`HTTPS_GET`, `HTTPS_POST`, `HTTPS_HEAD`, `HTTPS_PUT`, `HTTPS_DELETE`, `HTTPS_PATCH`.

### Error codes

`HTTPS_ERROR_NONE`, `HTTPS_ERROR_BAD_URL`, `HTTPS_ERROR_TLS_HANDSHAKE`, `HTTPS_ERROR_NO_SOCKET`, `HTTPS_ERROR_CANT_CONNECT`, `HTTPS_ERROR_SEND_FAIL`, `HTTPS_ERROR_CONTENT_TOO_BIG`, `HTTPS_ERROR_TIMEOUT`, `HTTPS_ERROR_POLICY_BLOCKED`, `HTTPS_ERROR_UNKNOWN`.

See [Error codes](error-codes.md) for the full mapping.

### Version

`HTTPS_SAMP_VERSION` is a string constant carrying the plugin version. It is generated from `Cargo.toml` at build time and matches the version printed in the startup banner.
