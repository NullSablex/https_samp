# Installation

The plugin ships as a single 32-bit shared library: `https_samp.so` on Linux and `https_samp.dll` on Windows. The same binary works on both SA-MP and Open Multiplayer.

## Files

The release artifact set is:

| File | Where it goes |
|---|---|
| `https_samp.so` or `https_samp.dll` | Server plugin directory. |
| `https_samp.inc` | Pawn compiler include path (typically `pawno/include/` or `qawno/include/`). |

The Pawn header is generated from `include/https_samp.inc.in` during the Rust build and copied into `dist/` when a release is packaged.

## Building from source

The plugin targets 32-bit because both servers are 32-bit. Two scripts are provided.

| Script | Host | Outputs |
|---|---|---|
| `scripts/build-linux.sh` | Linux | `dist/https_samp.so` and, via `cargo-xwin`, `dist/https_samp.dll`. |
| `scripts/build-windows.sh` | Windows (Git Bash) | `dist/https_samp.dll` and, via WSL or Docker/cross, `dist/https_samp.so`. |

Both scripts install missing Rust targets and tooling on demand and accept the `PROFILE` environment variable (`release` is the default).

## SA-MP

1. Copy the platform-specific binary into the server's `plugins/` directory.
2. Add the plugin name to `server.cfg`. On Linux the entry is `plugins https_samp`; on Windows it is `plugins https_samp.dll`.
3. Copy `https_samp.inc` to the Pawn compiler include directory and add `#include <https_samp>` to the gamemode or filterscript that uses it. The header is identical on SA-MP and Open Multiplayer.
4. Recompile the gamemode and restart the server.

Open Multiplayer running in legacy mode behaves identically to SA-MP for installation purposes.

## Open Multiplayer (native mode)

Open Multiplayer auto-discovers components placed in the server's components folder and loads them through `ComponentEntryPoint` without any configuration entry. The plugin then has access to the server's component lifecycle (`on_omp_ready`, `on_component_free`) in addition to the SA-MP-style callbacks.

1. Drop the binary into the Open Multiplayer components folder. No `config.json` entry is needed for native mode.
2. The component UID is declared in the plugin metadata and persisted across builds, so no manual configuration is needed.
3. Copy the include file as in the SA-MP setup and recompile.

## Open Multiplayer (legacy mode)

Legacy mode is opt-in and only useful if you want to force the SA-MP compatibility path on Open Multiplayer (for example, to mirror a SA-MP setup exactly).

1. Drop the binary into the plugins folder rather than the components folder.
2. Add the binary name to `pawn.legacy_plugins` in `config.json`.
3. Copy the include file as in the SA-MP setup.

The plugin logs which mode it loaded into on startup. Look for either `running on native Open Multiplayer` or `running on SA-MP (or Open Multiplayer legacy mode)` to confirm the active path.

## Verifying the installation

After the server boots, the startup log shows a banner with the plugin name, version, build date, and repository. If the banner is missing the binary was not loaded — re-check the file name in `server.cfg` and that the binary architecture matches the server.

Sending a trivial GET against a known endpoint from inside `OnGameModeInit` is enough to confirm that the worker pool, the response queue, and the callback dispatch are wired together. The `examples/` folder contains a self-contained snippet for this purpose.

## Uninstalling

Remove the entry from `server.cfg`, delete the binary from the plugin directory, and remove the include file. No configuration files are written by the plugin itself.
