# https_samp

HTTPS client plugin for SA-MP and Open Multiplayer, written entirely in Rust. Non-blocking requests, transparent decompression, automatic callback dispatch, secure-by-default transport, and mutual TLS support. The same compiled binary loads on SA-MP and on Open Multiplayer — natively as a component (recommended) or via legacy mode. No external runtime dependencies.

## Where to start

| Goal | Path |
|---|---|
| First time here | [Installation](installation.md) → [Usage](usage.md) → [API reference](api-reference.md) |
| Sending a request | [Usage](usage.md) → [Body builders](body-builders.md) |
| Hardening | [Security](security.md) → [Error codes](error-codes.md) |
| Migrating | [Migration](migration.md) |
| Looking for runnable examples | the `examples/` folder in the repository |

## Highlights

- **Single binary, two runtimes.** Loads on SA-MP via the legacy plugin ABI and on Open Multiplayer as a native component. The plugin detects the environment at load time and uses whichever lifecycle the host server provides.
- **Non-blocking.** Requests are dispatched to a worker thread pool sized between two and eight workers depending on the host CPU. The Pawn caller returns immediately; the response is delivered later through a callback.
- **No Pawn timer required.** The unified tick provided by the rust-samp v3 runtime drains the response queue automatically on both servers. The `https_process_queue` native is kept only for backwards compatibility and is a no-op.
- **Secure by default.** rustls-based TLS, no system proxy is honoured, HTTP-to-HTTPS downgrades are refused, cross-host redirects require explicit opt-in, and the `Authorization` header is stripped when crossing hosts.
- **Bounded resource usage.** Response queue capped at 1024 entries (oldest dropped on overflow); response body capped per call (default 64 KiB, range 4 KiB to 1 MiB); PEM file loader capped at 256 KiB.
- **Body builders.** Raw, JSON-validated, and form-urlencoded payloads can be staged ahead of a POST without packing them into the call site.
- **Mutual TLS.** A client certificate identity can be installed from a PEM buffer or from a file and is applied transparently to subsequent requests.

## Project layout

| Path | Purpose |
|---|---|
| `dist/` | Compiled artifacts produced by the build scripts. |
| `include/https_samp.inc` | Pawn header generated at build time from `include/https_samp.inc.in`. |
| `examples/` | Runnable Pawn examples (one file per scenario). |
| `scripts/` | Build helpers for Linux and Windows hosts. |
| `src/` | Rust source for the plugin. |
| `docs/` | This documentation. |
