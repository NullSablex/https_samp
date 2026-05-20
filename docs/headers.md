# Headers

Outgoing requests compose their header set from three sources, evaluated in order of increasing precedence: the built-in `User-Agent`, the global header table, and the temporary header table. When the same name appears in more than one layer, the later layer overrides the earlier one.

## The built-in User-Agent

Every request starts with `User-Agent: SA-MP HTTPS/1.0`. This default is applied before any user-supplied header, which means it can be overridden by adding a `User-Agent` entry to either the global or the temporary table. The default is intentionally generic and stable across versions so that remote services can identify traffic from this plugin without depending on the version string.

## Global headers

Global headers are added with `https_set_global_header` and persist across every subsequent request until explicitly cleared. They are appropriate for values that do not change between calls within a session — an authentication bearer token used against a single backend, a fixed `Accept-Language`, a deployment tag.

Adding a global header with a key that already exists replaces the previous value. The table can be wiped entirely with `https_clear_global_headers` and individual entries can be supplanted with a temporary header.

## Temporary headers

Temporary headers are added with `https_set_header` and apply only to the next request. After the response is enqueued — regardless of whether the request succeeded, failed, was blocked by policy, or timed out — the temporary table is cleared automatically. This means each request starts from a known baseline and there is no need to track "did I clear that header" manually.

A temporary header takes precedence over a global header with the same name for that one request. This is the right place for per-call values such as `Idempotency-Key`, a request-scoped `X-Trace-Id`, or a `Content-Type` that differs from the default the body builders would have set.

## Header ordering and casing

The plugin does not normalize header names. Whatever string you supply is what goes on the wire, byte-for-byte, including case. If you care about how a particular service logs your headers, use the canonical form (`Content-Type`, not `content-type` or `CONTENT-TYPE`).

The relative order of headers within a single request is not guaranteed and should not be relied upon — HTTP treats header order as insignificant except for duplicated names, which the plugin does not produce.

## Interaction with the body builders

The body builders set a `Content-Type` automatically when they imply one:

- `https_jsonf` installs `application/json; charset=utf-8` as a temporary header on success.
- `https_form_add` installs `application/x-www-form-urlencoded` as a temporary header on each successful append.
- `https_bodyf` does not set any Content-Type; you are responsible for adding one if the body is not plain text.

Because the builders use the temporary table, a manual `https_set_header("Content-Type", ...)` placed after the builder call will override the builder's choice for that one request. A manual call placed before the builder will be overridden by the builder. Order matters here.

When a POST request is dispatched with the inline `data` argument empty and a builder payload is consumed, the corresponding `Content-Type` is also moved into the temporary table just before the request is built, so the on-the-wire result is consistent with the builder semantics.

## Authorization and cross-host redirects

If the request is allowed to cross to a different host via `https_allow_cross_host_once` and a redirect actually crosses hosts, the `Authorization` header is stripped from the follow-up request. This is a fixed policy: credentials are scoped to the origin you addressed, never silently forwarded elsewhere. The comparison is case-insensitive in the header name and unconditional. See [Security](security.md) for the full redirect rules.
