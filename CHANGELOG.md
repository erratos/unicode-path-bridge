# Changelog

## v0.1.0 — 2026-04-27

Project renamed from **Unicode Path Bridge (UBP)** to **eupb — Explorer
Unicode Path Bridge**. Complete rewrite in Rust; the original C# /
.NET Framework 4.x implementation is preserved under `archive/dotnet/`.

### Added

- Rust implementation targeting `x86_64-pc-windows-msvc`, compiled as a
  Windows-subsystem binary (no console window created).
- CLI parser (`clap`):
  - `--hide-console` / `--show-console` (hide is default)
  - `--wait-exit` (alias `--wait`) / `--no-wait` (**no-wait is default** —
    eupb launches the target and returns 0 immediately; use `--wait-exit`
    to propagate the target's exit code)
  - `--cwd <DIR>` for the target's working directory
  - `--show-errors` / `--quiet-errors` (show is default); both the bare
    flag and the `--quiet-errors=<value>` form are recognised before clap
    parses, so early errors are also suppressed
  - `--log <FILE>` invocation log, UTF-8 with BOM for Notepad
  - `--set-env NAME=VALUE` (repeatable) to set or override environment
    variables for the target. Values are planted before `CreateProcessW`
    so shells that re-parse their `-Command` string (PowerShell's `&`,
    `'`, `$`, `;`…) never see them. Parent environment is inherited;
    overrides are case-insensitive per Windows semantics.
  - `--canonicalize` (off by default) — resolves symlinks and normalises
    dots. Disabled by default because it adds a `\\?\` extended-length
    prefix that breaks Electron apps (VS Code, VSCodium, …).
- Proper Microsoft C/C++ command-line argument escaping, validated by a
  `CommandLineToArgvW` round-trip test.
- `CreateProcessW` launch with `CREATE_NO_WINDOW` and
  `CREATE_UNICODE_ENVIRONMENT`; `DETACHED_PROCESS` in `--no-wait` mode;
  exit code propagation via `WaitForSingleObject` + `GetExitCodeProcess`
  in `--wait-exit` mode.
- `std::panic::set_hook` routes panics to a `MessageBoxW` so internal
  errors are visible even without a console window.
- PATH + PATHEXT resolution for target programs.
- Structured exit codes: 0 success, 1 usage, 2 target-not-found,
  3 CreateProcess failure, 4 wait / exit-code failure.
- Embedded application manifest: `longPathAware`, Win10/11
  `supportedOS`, `asInvoker`, UTF-8 `activeCodePage`.
- Test helper binary `eupb-test-target.exe` used by integration tests.
- 29 unit tests (`tests/escape.rs`, table-driven via `rstest`) and
  52 integration tests (`tests/integration.rs`) covering ASCII, French
  accents, apostrophes, Cyrillic, CJK, emoji, trailing backslash + space,
  UNC long paths, `--no-wait`, `--cwd`, `--log`, PATH resolution, and
  a full `--set-env` matrix (Unicode, shell metacharacters, multiple
  overrides, case-insensitive replacement, parent-env preservation,
  error paths).
- 10 ready-to-import `.reg` templates in `examples/`.

### Known behaviour

- **`Set-Clipboard` and clipboard writes require `--wait-exit`.**
  The default `--no-wait` launches the child as `DETACHED_PROCESS`, which
  has no message loop. `Set-Clipboard` silently does nothing in that
  context. See `examples/copy-path.reg` for the pattern.

### Archived

- `archive/dotnet/` — the original C# sources, PowerShell test harness,
  build script, manifest, and `PLAN.md`.

## v0.8.0 — 2026-04-07 (archived, .NET Framework)

Initial release as **Unicode Path Bridge (UBP)**, C# / .NET Framework 4.x.

- Unicode-safe argument forwarding from Explorer context menu.
- No console window flash (`/target:winexe`).
- Microsoft command-line argument escaping (spaces, quotes, backslashes).
- Long path support via application manifest.
- Error dialogs for missing target or launch failure.
- PATH resolution.
- Example `.reg` files.
- PowerShell test suite for Unicode paths.
