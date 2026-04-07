# Unicode Path Bridge (UBP)

A tiny, zero-dependency Windows utility that passes file paths from Explorer's right-click context menu to any program — with **full Unicode support** and **no console window flash**.

## The Problem

When you add a custom right-click menu entry in the Windows registry to run a script (PowerShell, Python, etc.), you hit a combination of issues:

| Problem | What happens |
|---------|-------------|
| **Unicode corruption** | PowerShell 5.1 converts non-ANSI characters (Cyrillic, CJK, emoji) to `?` |
| **8.3 path format** | Using `%1` instead of `%V` can give you `C:\DOSSIE~1\FILE.TXT` |
| **Console flash** | A black command prompt window flickers for ~0.5s on every click |
| **VBScript workaround fails** | Using `wscript.exe` to hide the window reintroduces ANSI corruption |

**UBP solves all four at once.** It's a single `.exe` compiled as a Windows application (no console), that receives arguments in Unicode from Explorer and forwards them to your target program.

## Quick Start

### 1. Build

```batch
build.cmd
```

Or manually:

```batch
C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe /target:winexe /win32manifest:src\ubp.manifest /out:ubp.exe src\ubp.cs
```

No SDK, no Visual Studio, no NuGet — just the .NET Framework 4.x compiler that's already on your machine.

### 2. Install

Copy `ubp.exe` to a permanent location (e.g., `C:\Tools\ubp.exe`).

### 3. Configure the Registry

Import one of the example `.reg` files from the [`examples/`](examples/) folder, or create your own entry:

```
"C:\Tools\ubp.exe" "powershell.exe" "-File" "C:\Scripts\my-script.ps1" "%V"
```

The `%V` token is replaced by Explorer with the full Unicode path of the file or folder you right-clicked.

## Usage Examples

```
:: Run a PowerShell script
"C:\Tools\ubp.exe" "powershell.exe" "-File" "C:\Scripts\process.ps1" "%V"

:: Run a Python script
"C:\Tools\ubp.exe" "python.exe" "C:\Scripts\process.py" "%V"

:: Open in any program
"C:\Tools\ubp.exe" "C:\MyApp\tool.exe" "--input" "%V"
```

See [`examples/`](examples/) for ready-to-import `.reg` files.

## How It Works

```
Explorer ──(%V)──► Registry ──(Unicode args)──► ubp.exe ──(Unicode args)──► Target Program
                                                   │
                                                   ├─ No console window (compiled as /target:winexe)
                                                   ├─ Correct argument escaping (spaces, quotes, backslashes)
                                                   └─ Long path support via manifest (>260 chars)
```

For a detailed technical explanation, see [docs/how-it-works.md](docs/how-it-works.md).

## Requirements

- Windows 10 or 11
- .NET Framework 4.x (pre-installed on all Windows 10/11 machines)

## Alternatives

| Tool | Trade-off |
|------|-----------|
| [PowerShell 7](https://github.com/PowerShell/PowerShell) | Fixes Unicode natively, but requires manual install (~400 MB) |
| [createprocess-windows](https://github.com/cubiclesoft/createprocess-windows) | Full-featured CreateProcess wrapper, but complex CLI and requires C++ compilation |
| [SharpShell](https://github.com/dwmkerr/sharpshell) | Proper COM shell extension framework, but heavyweight for simple forwarding |
| VBScript (`wscript.exe`) | Hides the window, but reintroduces ANSI encoding corruption |

UBP is for you if you want a **single file, zero config, zero dependency** solution.

## Testing

```powershell
powershell -ExecutionPolicy Bypass -File tests\test-paths.ps1
```

This runs the bridge with French accents, Cyrillic, Japanese, emoji, and edge-case paths, then verifies the target script received them correctly.

## License

[MIT](LICENSE)
