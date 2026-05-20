# Changelog

All notable changes to this project are documented in this file.

Format inspired by [Keep a Changelog](https://keepachangelog.com/). Versioning follows [Semantic Versioning](https://semver.org/). Older entries live under [`changelog/`](changelog/).

## [1.0.0] — 2026/05/20

Initial public release. Built on top of [rust-samp v3.0.0](https://github.com/NullSablex/rust-samp/releases/tag/v3.0.0). A single `.so` / `.dll` binary runs on SA-MP and on Open Multiplayer (native component or legacy mode).

### Added

- **Request submission.** `https(index, type, url, data, callback)` issues non-blocking requests. Supported methods: `HTTPS_GET`, `HTTPS_POST`, `HTTPS_HEAD`, `HTTPS_PUT`, `HTTPS_DELETE`, `HTTPS_PATCH`. The Pawn callback receives `(index, response, status, error)`.
- **Worker pool.** Two to eight background threads consume jobs from a bounded queue. A submission that meets a saturated pool falls back to a one-off thread so the caller never observes back-pressure.
- **Automatic callback dispatch.** The unified `on_tick` provided by rust-samp v3 drains the response queue on both runtimes — SA-MP via `ProcessTick`, Open Multiplayer native via `ITimersComponent`. No Pawn timer is required. `https_process_queue` is retained as a no-op for backwards compatibility.
- **Header layers.** `https_set_header` (temporary, cleared after each request), `https_set_global_header` (persistent across requests), and `https_clear_global_headers`. A built-in `User-Agent` is set automatically and can be overridden by either layer.
- **Response headers in Pawn.** `https_response_header(key, dest, max_len)` reads any header of the response currently being delivered to its callback. Case-insensitive lookup.
- **Cancellation.** `https_cancel(index)` drops a pending response so its callback is never delivered. Useful when the script-side context (e.g. a player session) is gone before the network call returns.
- **Per-request total timeout.** `https_set_timeout_once(total_ms)` overrides the default twelve-second request timeout for the next call only.
- **Body builders, mutually exclusive and one-shot.** Each successfully staged payload replaces any previous one and is consumed by the next POST / PUT / PATCH whose inline body is empty.
  - `https_bodyf(data)` — raw bytes.
  - `https_jsonf(data)` — JSON-validated payload; sets `Content-Type: application/json; charset=utf-8`.
  - `https_form_add(key, value)` — form-urlencoded accumulator; sets `Content-Type: application/x-www-form-urlencoded`.
  - `https_multipart_add_text(key, value)` and `https_multipart_add_file(field, filename, path)` — `multipart/form-data` builder for file uploads. The plugin picks the boundary and sets the matching `Content-Type` automatically.
- **Authentication helpers (one-shot, temporary header).** `https_set_basic_auth_once(user, password)` (base64-encoded `user:password`) and `https_set_bearer_once(token)`.
- **Cookies, opt-in.** `https_cookies_enable(bool)` toggles an in-memory cookie jar that persists across requests for the duration of the plugin's lifetime. `https_cookies_clear()` replaces the jar with a fresh, empty one.
- **Mutual TLS.** `https_mtls_set_pem(pem)` and `https_mtls_set_pem_file(path)` install a client certificate identity from a combined PEM blob (cert + key). The PEM file loader refuses files larger than 256 KiB. `https_mtls_clear()` removes the identity.
- **Cross-host redirect opt-in.** `https_allow_cross_host_once(bool)` consents to a single cross-host redirect for the next request; without it, cross-host redirects fail with `HTTPS_ERROR_POLICY_BLOCKED`. The `Authorization` header is stripped automatically when a redirect crosses hosts.
- **Body-size cap.** `https_set_max_body_bytes(bytes)` and `https_get_max_body_bytes()` set the maximum response body size per request. Clamped to the inclusive range 4 KiB – 1 MiB; default 64 KiB. Overflow returns `HTTPS_ERROR_CONTENT_TOO_BIG` with the HTTP status preserved.
- **Queue introspection.** `https_queue_len()` reports the number of responses pending dispatch.
- **Pawn header.** `https_samp.inc` declares all natives, request type constants (`HTTPS_*`), and error codes (`HTTPS_ERROR_*`). Generated at build time from `include/https_samp.inc.in` so the embedded `HTTPS_SAMP_VERSION` always tracks `Cargo.toml`.
- **Documentation.** MkDocs Material site under `docs/` covering installation, usage, headers, body builders, security, cookies, error codes, API reference, and migration. Hosted with `actions/deploy-pages`.
- **Examples.** Self-contained `.pwn` snippets under `examples/` covering simple GET, JSON POST, form POST, header layering, cross-host redirects, mTLS, REST methods, response headers, cancellation and per-request timeouts, multipart uploads, auth helpers, and cookie-driven sessions.
- **CI.** GitHub Actions workflows for build/test/clippy (`rust.yml`), MkDocs site deploy (`docs.yml`), and release artifact upload with a tag-vs-`Cargo.toml` sanity check and auto-generated release notes pinning the rust-samp SDK version (`release.yml`).
- **Build tooling.** `scripts/build-linux.sh` (Linux + Windows from Linux via `cargo-xwin`) and `scripts/build-windows.sh` (Windows + Linux via WSL or Docker/cross). A `build.rs` injects build date/time/year as compile-time env vars and renders the Pawn include from its template.

### Security defaults

- TLS via rustls with the `ring` crypto provider. WebPKI roots are statically linked; no system trust store is consulted; the system proxy environment variables are explicitly ignored.
- Connect timeout 7 seconds, request timeout 12 seconds (default; per-request override available).
- Manual redirect handling, maximum five hops.
- Anti-downgrade: once any step of a redirect chain used HTTPS, no subsequent step may use HTTP.
- Cross-host redirects refused unless explicitly enabled; `Authorization` stripped on cross-host hops.
- Pending response queue capped at 1024 entries; overflow drops the oldest entry.
- PEM file loader capped at 256 KiB.

### License

Distributed under the GNU Affero General Public License v3.0 or later.
