# Error codes

The fourth argument delivered to every callback is the error code. Zero means the request reached the remote server and produced an HTTP response; the status code in the third argument is whatever the server returned, and the body is whatever it sent. Any non-zero value means the request did not complete normally — the status will be zero (except for `CONTENT_TOO_BIG`, where the status received before the body was discarded is preserved) and the body will be empty.

The codes mirror the constants declared in `https_samp.inc`.

| Symbol | Value | Meaning |
|---|---:|---|
| `HTTPS_ERROR_NONE` | 0 | Request completed at the transport layer. The HTTP status is whatever the server returned. |
| `HTTPS_ERROR_BAD_URL` | 1 | The submitted URL did not parse, used an unsupported scheme, or the underlying client refused to build a request from it. |
| `HTTPS_ERROR_TLS_HANDSHAKE` | 2 | A connection-level failure that was not classifiable as a more specific I/O error. In practice this is the bucket for TLS negotiation failures. |
| `HTTPS_ERROR_NO_SOCKET` | 3 | The local network stack reported that the destination is unreachable or the local address is not available. |
| `HTTPS_ERROR_CANT_CONNECT` | 4 | The TCP connection attempt was refused or could not be established for an otherwise unclassified reason. |
| `HTTPS_ERROR_SEND_FAIL` | 5 | The connection was open but writing the request failed (broken pipe, premature shutdown). |
| `HTTPS_ERROR_CONTENT_TOO_BIG` | 6 | The response body exceeded the current value of `https_get_max_body_bytes`. The HTTP status is preserved in this case. |
| `HTTPS_ERROR_TIMEOUT` | 7 | One of the timeouts (connect or request) fired before the operation completed. |
| `HTTPS_ERROR_POLICY_BLOCKED` | 8 | The request was rejected by the plugin's own policy, not by the network or the remote server. See below. |
| `HTTPS_ERROR_UNKNOWN` | 10 | The error did not match any of the above categories. Treat as a transient failure unless it recurs deterministically. |

## When POLICY_BLOCKED is raised

`HTTPS_ERROR_POLICY_BLOCKED` is the plugin telling the caller "I refused to do this, the remote service was never asked." It is raised in the following situations:

- A redirect chain exceeded five hops.
- A `3xx` response had no `Location` header or had a `Location` that did not resolve against the current URL.
- A redirect would have downgraded the connection from HTTPS to HTTP at any step.
- A redirect would have crossed to a different host and `https_allow_cross_host_once(true)` was not set immediately before the request.

`POLICY_BLOCKED` is not transient. Retrying the same request without changing the configuration will produce the same result. The fix is to either correct the URL on the script side or relax the policy explicitly (only the cross-host case is configurable; downgrade and hop-count limits are fixed).

## When BAD_URL is raised

`HTTPS_ERROR_BAD_URL` reflects three different conditions, all detected before any network activity:

- The URL does not parse at all.
- The URL parses but uses a scheme other than `http://` or `https://`.
- The HTTP client refused to build a request, typically because the URL has a structural problem the parser accepted but the request builder did not.

Inspect the URL on the script side before retrying. The plugin does not attempt to canonicalize, fix up, or escape the input.

## When CONTENT_TOO_BIG is raised

The plugin reads the response body until the cap is reached, then stops. If the read produced more bytes than the cap allows, the body is discarded and `HTTPS_ERROR_CONTENT_TOO_BIG` is reported with the HTTP status that was received. This is the only failure case where the status code in the callback is non-zero — all other non-zero errors mean no HTTP response was processed and the status is also zero.

If a particular endpoint legitimately returns more than the default 64 KiB, raise the cap with `https_set_max_body_bytes` before the call. The cap is global, not per-request, so remember to lower it back if the rest of the application benefits from the tighter default.

## When TIMEOUT is raised

Both timeouts share the same error code. The connect timeout (seven seconds) covers TCP establishment and the TLS handshake; the request timeout (twelve seconds) covers everything, including body read. Timeouts are not configurable from Pawn — see [Security](security.md) for the rationale.

## Distinguishing transport failures

The buckets `NO_SOCKET`, `CANT_CONNECT`, `SEND_FAIL`, and `TLS_HANDSHAKE` reflect distinct categories of low-level failure, but the line between them depends on the exact `ErrorKind` reported by the operating system. For diagnostic purposes treat them as a coarse classification rather than a precise diagnosis. If you need to distinguish them programmatically — for example, to decide between retrying immediately and backing off — the values above are stable across plugin versions.
