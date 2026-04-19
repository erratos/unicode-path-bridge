# eupb — Explorer Unicode Path Bridge

A single-file Windows wrapper that launches a target program with
**Unicode-safe arguments** and **no console window flash**. Built for
use from the Explorer right-click menu, but works anywhere.

## The problem it solves

When a Windows registry context menu entry runs a script, four things
go wrong:

| Problem | Effect |
|---|---|
| PowerShell 5.1 converts args through the ANSI code page | `C:\Тест\файл.txt` → `C:\???\????.txt` |
| `%1` in the registry can give 8.3 short paths | `C:\Dossier Été\` → `C:\DOSSIE~1\` |
| Console programs flash a black window on every click | ~50–200 ms flicker, ugly |
| `wscript.exe` hides the window but reintroduces ANSI corruption | back to `?` characters |

`eupb.exe` is a GUI-subsystem executable that receives its arguments in
UTF-16 (via `CommandLineToArgvW`), re-escapes them per the Microsoft
C/C++ command-line rules, and launches the target via `CreateProcessW`
with `CREATE_NO_WINDOW`. The arguments survive end-to-end.

## Usage

```
eupb [OPTIONS] -- <TARGET> [TARGET_ARGS...]
```

The `--` is recommended whenever the target or its args may look like
eupb options.

### Options

| Option | Default | Effect |
|---|---|---|
| `--hide-console` | on | Launch the target with `CREATE_NO_WINDOW` |
| `--show-console` | | Keep the target's console visible |
| `--wait` | on | Wait for the target to exit, propagate its exit code |
| `--no-wait` | | Launch detached and return 0 immediately |
| `--cwd <DIR>` | inherit | Working directory for the target |
| `--show-errors` | on | Show a MessageBox on launch errors |
| `--quiet-errors` | | Suppress error dialogs; use exit codes only |
| `--log <FILE>` | | Log invocation details (UTF-8 + BOM) |
| `--version`, `-V` | | Show version |
| `--help`, `-h` | | Show help |

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success (or target's exit code when `--wait`) |
| 1 | Usage error (no target) |
| 2 | Target program not found |
| 3 | `CreateProcessW` failed |
| 4 | Wait / exit-code retrieval failed |

## Registry example

Add "Run my script" to the right-click menu for files:

```reg
Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\*\shell\MyScript]
@="Run my script"
"Icon"="C:\\Tools\\eupb.exe"

[HKEY_CLASSES_ROOT\*\shell\MyScript\command]
@="\"C:\\Tools\\eupb.exe\" -- \"powershell.exe\" \"-NoProfile\" \"-File\" \"C:\\Scripts\\my-script.ps1\" \"%V\""
```

See [`examples/`](examples/) for ready-to-import `.reg` files.

## Build

```
cargo build --release
```

The release binary lands at `target\release\eupb.exe`. The application
manifest (long-path awareness, Win10/11, `asInvoker`, UTF-8 active code
page) is embedded automatically by `build.rs`.

## Test

```
cargo test
```

Runs 29 unit tests for `escape_arg` (table-driven, including a
`CommandLineToArgvW` round-trip) and 16 integration tests spawning
`eupb.exe → eupb-test-target.exe` across ASCII, Cyrillic, CJK, emoji,
French accents, apostrophes, trailing-backslash-plus-space, UNC long
paths, `--no-wait`, `--cwd`, `--log`, and PATH+PATHEXT resolution.

## Requirements

- Windows 10 22H2 / Windows 11
- Rust 1.78+ with the `x86_64-pc-windows-msvc` target (for building)

## History

Version 0.8.0 was a .NET Framework 4.x / C# implementation (project
name: "UBP"). The C# sources are preserved under
[`archive/dotnet/`](archive/dotnet/). Version 0.1.0 onward is a Rust
rewrite with the new project name **eupb** (Explorer Unicode Path
Bridge).

## License

[MIT](LICENSE)
