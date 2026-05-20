# Usage

This page covers the request lifecycle: how a Pawn script submits an HTTPS call, how the response flows back, and how the per-request state is composed.

## The request model

Every request is described by five values:

- An **index**, an arbitrary integer chosen by the caller. The plugin echoes it back to the callback verbatim and never inspects it. Use it to correlate the response with the originating script-side state (a player id, a request slot, a queue ticket, anything).
- A **request type**, expressed by the symbolic constants `HTTPS_GET`, `HTTPS_POST`, `HTTPS_HEAD`, `HTTPS_PUT`, `HTTPS_DELETE`, and `HTTPS_PATCH`.
- A **URL**. Both `http://` and `https://` schemes are accepted at the entry point, but the redirect policy will refuse any downgrade from HTTPS to HTTP later on.
- A **body**. For GET and HEAD the body is ignored. For POST the inline string is used if non-empty; otherwise the plugin consumes whatever was prepared by the body builders (see [Body builders](body-builders.md)).
- A **callback name**, a Pawn `public` to be invoked when the response is ready.

The native returns immediately after queueing the job. There is no return value beyond a success indicator; the actual outcome arrives later via the callback.

## The callback

The callback declared in Pawn must accept four arguments, in this order:

- The **index** that was supplied at submission time.
- The **response body** as a string. For HEAD requests, or any failure path, this string is empty. The body is read as bytes and presented as UTF-8; non-UTF-8 sequences are replaced rather than rejected.
- The **HTTP status code**. Zero is used as a sentinel when the request never produced an HTTP response (transport failure, policy block, malformed URL, timeout).
- The **error code**. Zero means success at the transport layer (the status code is whatever the server returned). Any non-zero value indicates that the request did not complete normally; consult [Error codes](error-codes.md) for the mapping.

Callbacks are dispatched on the server thread, from inside the unified tick provided by the runtime. There is no Pawn timer to schedule and no native to poll; the plugin drives this automatically on both SA-MP and Open Multiplayer.

## Per-request headers

Two layers of headers contribute to every outgoing request, in order:

1. A built-in `User-Agent`. It is set before any user-supplied header and can be overridden.
2. The **global headers**, added with `https_set_global_header`. These persist across requests until explicitly cleared.
3. The **temporary headers**, added with `https_set_header`. These apply only to the next request and are cleared automatically when the response is enqueued, regardless of whether the request succeeded.

When the same header name is set in more than one layer, the later layer wins. Header keys are not normalized; supply them in the canonical case (`Content-Type`, not `content-type`) if you care about how the remote server logs them.

## Concurrency model

Requests are processed by a worker thread pool sized between two and eight threads depending on the host. The submission queue is bounded; when full, the offending request is dispatched on a one-off thread instead of being dropped, so call sites never see a synchronous failure due to back-pressure.

Responses are kept in a separate bounded queue. If the queue overflows (more than 1024 pending responses), the oldest entry is dropped to make room. This is a safety net for pathological cases — under normal operation the unified tick drains responses every server frame, far faster than they accumulate.

## Method semantics

- `HTTPS_GET` and `HTTPS_DELETE` ignore any prepared body and never send one. `DELETE` with a body is uncommon enough that the plugin does not surface it; if a particular API requires it, use a different method.
- `HTTPS_POST`, `HTTPS_PUT`, and `HTTPS_PATCH` use the inline `data` argument when it is non-empty; otherwise they consume the one-shot payload staged by `https_bodyf`, `https_jsonf`, `https_form_add`, or the multipart builder, and set the corresponding `Content-Type` automatically if the builder implies one.
- `HTTPS_HEAD` issues a `HEAD` and returns only the status code. The body is never read; the response string is always empty. Redirects on HEAD are not followed: the 3xx status is delivered as-is.

## Redirects

For GET and POST, up to five redirects are followed manually under a strict policy. The policy is detailed in [Security](security.md); the short version is that downgrades and unexpected host changes are refused outright, and method downgrades on 301/302/303 follow the conservative interpretation that turns the follow-up into a GET without a body.

## Reading response headers

The callback receives the body, status, and error code as parameters, but the response headers are also available — inside the callback only — through `https_response_header(key, dest, max_len)`. Header names are matched case-insensitively (HTTP header names are not case-sensitive on the wire). Calling the native outside of a callback returns `false` and writes an empty string into the buffer.

The headers map is bound to the specific response currently being delivered, so reading it from inside the callback always reflects the response that triggered the call. The map is cleared when the callback returns.

## Cancelling a pending request

`https_cancel(index)` marks an index as cancelled. If the worker thread has not yet enqueued the response, the result is discarded at enqueue time; if the response is already queued waiting for dispatch, it is discarded at dispatch time. The network request itself may still complete in the background — the plugin only suppresses the callback. Calling cancel for an unknown or already-delivered index is a no-op.

## Per-request timeout

`https_set_timeout_once(total_ms)` overrides the default twelve-second request timeout for the next call only. The flag is consumed at submission time, so each request that needs a custom timeout must reset it. The connect timeout cannot be overridden per request; only the total request timeout is configurable.

## Reading the body

Response bodies are read with a hard cap controlled by `https_set_max_body_bytes`. The cap defaults to 64 KiB and is clamped to the inclusive range 4 KiB to 1 MiB. Exceeding the cap aborts the read and reports `HTTPS_ERROR_CONTENT_TOO_BIG` with the HTTP status that was received before the body was discarded.

Decompression is automatic for `gzip`, `brotli`, `deflate`, and `zstd` when the server advertises them; the cap applies to the decompressed size, which is what your script actually sees.
