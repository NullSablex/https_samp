# Migration

This page covers what changes when upgrading existing Pawn scripts that already use `https_samp`. Two transitions are worth calling out: the move from any older binary to the current one, and the move from any earlier Rust SDK to rust-samp v3 (which is the runtime the current release is built against).

## From an older https_samp to this release

### Native names and signatures

All native names are preserved. Existing scripts that reference `https`, `https_set_header`, `https_set_global_header`, `https_clear_global_headers`, `https_process_queue`, `https_queue_len`, `https_set_max_body_bytes`, `https_get_max_body_bytes`, `https_allow_cross_host_once`, `https_bodyf`, `https_jsonf`, `https_form_add`, `https_mtls_set_pem`, `https_mtls_set_pem_file`, and `https_mtls_clear` continue to compile and behave the same.

Several setters now declare a `bool:` return tag in the include file (success or failure). The on-the-wire ABI is unchanged — the tag is informational. Pawn code that already ignored the return value is unaffected; code that stored it in an untagged integer will see a tag-mismatch warning that can be silenced by tagging the destination variable.

### Include file name

The include file is now `https_samp.inc`, replacing the previous `a_https.inc`. Rename the `#include` line in any gamemode or filterscript that references the old name. The header is generated from `include/https_samp.inc.in` at build time; the previous file is no longer shipped.

### The Pawn timer is no longer needed

In earlier versions the response queue had to be drained by calling `https_process_queue` from a Pawn timer or from `OnGameModeInit` with `SetTimer`. This is no longer required. The unified tick provided by rust-samp v3 drains the queue automatically on every server frame on SA-MP and on every five-millisecond timer interval on native Open Multiplayer.

Existing scripts that still call `https_process_queue` continue to work. The native is preserved for backwards compatibility and is now a no-op; remove the timer and the call when convenient.

### Constants moved to enums

The request types and error codes are now declared inside Pawn `enum` blocks in the include file instead of as loose `#define`s. The symbol names are identical (`HTTPS_GET`, `HTTPS_ERROR_BAD_URL`, etc.) and the values are unchanged, so any code that uses them by name keeps working.

## From the legacy plugin lifecycle to the rust-samp v3 runtime

This is relevant if you build the plugin yourself or maintain a fork.

### Single binary, two ABIs

The current build emits both the SA-MP plugin exports and the Open Multiplayer native component entry point from the same binary. Native Open Multiplayer support is the default; the `samp-only` feature is not declared in this project, so every release behaves as a first-class component when loaded by an Open Multiplayer server and as a regular plugin on SA-MP.

### Component metadata

The component UID is stored in `[package.metadata.samp]` in `Cargo.toml` and persists across builds. Manual UID management is not required and is not exposed to Pawn.

### Lifecycle hooks

The plugin implements the unified hooks: `on_load`, `on_unload`, `on_amx_load`, `on_amx_unload`, `on_tick`, plus the Open Multiplayer-specific `on_omp_ready` and `on_component_free`. None of these are visible from Pawn — they exist only in the Rust side and they are why the response queue no longer needs a Pawn timer.

### Logger

Log output is routed through the runtime's logger (`samp::log`) with a `[https_samp]` prefix on every line emitted by the plugin (the decorative startup banner is intentionally left unprefixed). On SA-MP the lines reach the standard server log; on Open Multiplayer they reach the component log.

## Things that did not change

- The four-argument callback signature `(index, response[], status, error)`.
- The redirect policy (anti-downgrade, cross-host opt-in, five-hop ceiling).
- The response body size cap (default 64 KiB, range 4 KiB to 1 MiB).
- The pending response queue size (1024).
- The connect and request timeouts (seven and twelve seconds).
- The mTLS interface — both natives still take a combined PEM buffer or path.
- The behavior of `https_allow_cross_host_once` (one-shot, consumed at submission).
