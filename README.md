# https_samp

> HTTPS client plugin for SA-MP and Open Multiplayer, written in Rust — by [NullSablex](https://github.com/NullSablex)

![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)
![SA-MP](https://img.shields.io/badge/SA--MP-0.3.7+-orange)
![Open Multiplayer](https://img.shields.io/badge/Open%20Multiplayer-native%20%26%20legacy-orange)
![Build](https://img.shields.io/badge/build-Linux%20%7C%20Windows-green)
![Architecture](https://img.shields.io/badge/arch-x86%20(32--bit)-lightgrey)
[![Release](https://img.shields.io/github/v/release/NullSablex/https_samp?label=download)](https://github.com/NullSablex/https_samp/releases/latest)

## Overview

**https_samp** is a modern HTTPS client plugin for SA-MP (San Andreas Multiplayer) and [Open Multiplayer](https://open.mp) (open.mp), written entirely in Rust. It provides a non-blocking REST-grade HTTP client with body builders, mutual TLS, optional cookie persistence, response headers exposed to Pawn, and a strict secure-by-default redirect policy.

The same binary loads on SA-MP and on Open Multiplayer — natively as a component (recommended) or via legacy mode.

### Highlights

- **Zero external dependencies.** rustls-based TLS and HTTP/2 are statically linked. No system libraries to install.
- **Non-blocking requests.** A bounded worker pool (2–8 threads) processes calls in parallel; the Pawn caller returns immediately and the callback is dispatched automatically by the unified runtime tick — no Pawn timer required.
- **REST-complete.** `GET`, `POST`, `HEAD`, `PUT`, `DELETE`, and `PATCH`.
- **Body builders.** Raw, JSON-validated, form-urlencoded, and full `multipart/form-data` with file uploads, all staged one-shot.
- **Response headers in Pawn.** Read any header of the response currently being delivered via `https_response_header`.
- **Cancellation.** `https_cancel(index)` drops a callback that the gamemode no longer needs (e.g., player disconnected before the response came back).
- **Authentication helpers.** One-shot `Basic` and `Bearer` token natives — the plugin handles the base64 encoding.
- **Cookies (opt-in).** Per-process cookie jar shared across requests, toggleable and clearable from Pawn.
- **Mutual TLS.** Client certificate identity from a PEM buffer or a file on disk.
- **Secure by default.** Anti-downgrade redirects (no HTTPS → HTTP), explicit cross-host opt-in, `Authorization` stripping on cross-host hops, bounded body and queue sizes, hard timeouts.
- **Universal binary.** Built on top of [rust-samp](https://github.com/NullSablex/rust-samp) v3.0.0; one `.so`/`.dll` runs on SA-MP and on Open Multiplayer (native component or legacy).

## Installation

1. Download the latest release for your platform:
   - `https_samp.so` (Linux i686)
   - `https_samp.dll` (Windows i686, MSVC ABI)
   - `https_samp.inc` (Pawn include, shared between SA-MP and Open Multiplayer)
2. Place the binary in the server's `plugins/` directory.
3. Copy `https_samp.inc` to your compiler's include folder:
   - **Windows:** `pawno/include/` or `qawno/include/`
   - **Linux:** `include/` (at the server root)
4. Register the plugin:
   - **SA-MP** — add to `server.cfg`:
     ```
     plugins https_samp.so
     ```
     (or `https_samp.dll` on Windows)
   - **Open Multiplayer (native, recommended)** — drop the binary into the server's components folder. No `config.json` entry is required; the server discovers and loads it through `ComponentEntryPoint` automatically, with access to `ICore`, `ITimersComponent`, and the other native APIs.
   - **Open Multiplayer (legacy)** — to force the SA-MP compatibility path instead, add the binary under `pawn.legacy_plugins` in `config.json`. Same binary, no extra build flags.

> [!IMPORTANT]
> No system TLS library is required. The plugin is self-contained.

## Quick start

```pawn
#include <a_samp>
#include <https_samp>

public OnGameModeInit()
{
    // Submit a non-blocking GET. The "1" is an arbitrary correlation id.
    https(1, HTTPS_GET, "https://api.github.com/zen", "", "OnZen");
    return 1;
}

forward OnZen(index, response[], status, error);
public OnZen(index, response[], status, error)
{
    if (error != HTTPS_ERROR_NONE)
    {
        printf("[zen] request failed: error=%d", error);
        return 1;
    }

    // Read any response header (case-insensitive) inside the callback.
    new ratelimit[16];
    https_response_header("X-RateLimit-Remaining", ratelimit);

    printf("[zen] status=%d remaining=%s body=%s", status, ratelimit, response);
    return 1;
}
```

More runnable snippets — POST with JSON, multipart uploads, cookies, mTLS, per-request timeout, cancellation — are available in [`examples/`](examples/). The plugin natives and the include file are identical on both runtimes.

## Documentation

The full plugin documentation lives in [docs/](docs/):

| Document | Contents |
|---|---|
| [Installation](docs/installation.md) | Setup, `server.cfg` / `config.json`, requirements |
| [Usage](docs/usage.md) | Request lifecycle, callback signature, concurrency model |
| [Headers](docs/headers.md) | Temporary vs global, precedence, interactions with body builders |
| [Body builders](docs/body-builders.md) | Raw, JSON, form, multipart — one-shot staging |
| [Cookies](docs/cookies.md) | Enabling, clearing, interactions with mTLS and redirects |
| [Security](docs/security.md) | TLS, redirect policy, cross-host, mTLS, resource caps |
| [Error codes](docs/error-codes.md) | Full mapping with disambiguation per category |
| [API reference](docs/api-reference.md) | Every native with arguments and return semantics |
| [Migration](docs/migration.md) | Upgrade notes from older versions and from older SDKs |

## Building from source

### Requirements

- Rust stable toolchain with the targets `i686-unknown-linux-gnu` and `i686-pc-windows-msvc`
- `cargo-xwin` for cross-compiling the Windows `.dll` from Linux (installed automatically by the script)
- No system libraries — the build is 100% Rust

### Development build

```bash
cargo build --target i686-unknown-linux-gnu
```

### Release build (Linux + Windows)

From Linux:

```bash
./scripts/build-linux.sh
```

From Windows (Git Bash):

```bash
./scripts/build-windows.sh
```

Both scripts produce `dist/https_samp.so` and `dist/https_samp.dll`, each with full SA-MP + Open Multiplayer native support.

> [!CAUTION]
> This plugin is distributed under the AGPL v3 (or later). Any derivative work — including services that expose modified versions to users over a network — must keep the source code open under the same license.

## License

Copyright (c) 2026 NullSablex

This project is licensed under the [GNU Affero General Public License v3.0 (or later)](LICENSE).
