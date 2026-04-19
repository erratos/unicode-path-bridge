# Changelog

## v0.1.0 — 2026-04-19

Project renamed from **Unicode Path Bridge (UBP)** to **eupb — Explorer
Unicode Path Bridge**. Complete rewrite in Rust; the original C# /
.NET Framework 4.x implementation is preserved under `archive/dotnet/`.

### Added

- Rust implementation targeting `x86_64-pc-windows-msvc`, compiled as a
  Windows-subsystem binary (no console window created).
- CLI parser (`clap`):
  - `--hide-console` / `--show-console` (hide is default)
  - `--wait` / `--no-wait` (wait is default)
  - `--cwd <DIR>` for the target's working directory
  - `--show-errors` / `--quiet-errors` (show is default)
  - `--log <FILE>` invocation log, UTF-8 with BOM for Notepad
- Proper Microsoft C/C++ command-line argument escaping, validated by a
  `CommandLineToArgvW` round-trip test.
- `CreateProcessW` launch with `CREATE_NO_WINDOW` and
  `CREATE_UNICODE_ENVIRONMENT`; exit code propagation via
  `WaitForSingleObject` + `GetExitCodeProcess` in `--wait` mode.
- PATH + PATHEXT resolution for target programs.
- Structured exit codes: 0 success, 1 usage, 2 target-not-found,
  3 CreateProcess failure, 4 wait / exit-code failure.
- Embedded application manifest: `longPathAware`, Win10/11
  `supportedOS`, `asInvoker`, UTF-8 `activeCodePage`.
- Test helper binary `eupb-test-target.exe` used by integration tests.
- 29 unit tests (`tests/escape.rs`, table-driven via `rstest`) and
  16 integration tests (`tests/integration.rs`) covering ASCII, French
  accents, apostrophes, Cyrillic, CJK, emoji, trailing backslash + space,
  UNC long paths, `--no-wait`, `--cwd`, `--log`, and PATH resolution.

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
