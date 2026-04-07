# How Unicode Path Bridge Works

## The Problem in Detail

### 1. Unicode Corruption in PowerShell 5.1

Windows Explorer internally handles all file paths as Unicode (UTF-16). When you right-click a file and trigger a registry command, Explorer passes the path as a Unicode string to `CreateProcess()`.

However, **PowerShell 5.1** (`powershell.exe`) is tied to the legacy console subsystem (`conhost.exe`). When it receives command-line arguments, it may convert them through the system's ANSI code page (e.g., Windows-1252 for Western European locales).

Characters that don't exist in the local code page are replaced with `?`. This means:
- `C:\Тестовая папка\файл.txt` → `C:\???????? ?????\????.txt`
- `C:\テスト\ファイル.txt` → `C:\???\????.txt`
- `C:\📁 Folder\📄 File.txt` → `C:\? Folder\? File.txt`

This is fixed in **PowerShell 7** (`pwsh.exe`), but it's not installed by default.

### 2. The 8.3 Short Name Trap

The `%1` token in registry commands is a legacy from MS-DOS. Depending on the registry key location and path length, Windows may substitute the short 8.3 path:

```
Expected: C:\Dossier Été 2026\Fichier.txt
Received: C:\DOSSIE~1\FICHIER.TXT
```

**Solution:** Always use `%V` (or `%L`) instead of `%1`. These tokens force the long path format.

### 3. Console Window Flash

Any `.exe` compiled as a console application (`/target:exe`) will briefly show a command prompt window when launched from Explorer. This is a Windows design behavior — the OS creates a console for the process before it even starts executing.

The classic workaround is to use VBScript (`wscript.exe`) as a launcher, since it's a GUI subsystem process. But VBScript processes strings through ANSI, bringing back the corruption from problem #1.

### 4. `-Command` vs `-File` Parsing

If the registry entry uses `-Command` instead of `-File`:

```
powershell.exe -Command "& 'C:\Scripts\process.ps1' 'C:\L'été\file.txt'"
```

The path is injected directly into PowerShell's parser. Apostrophes in file names (`L'été`) break the string quoting, spaces cause argument splitting, and the script either fails or processes the wrong path.

Using `-File` avoids this because the path is passed as a pre-parsed argument, not as source code to evaluate.

## The Solution

UBP is a C# program compiled as a **Windows application** (`/target:winexe`), which means:

1. **No console window is created** — the OS treats it as a GUI application
2. **Arguments are received as Unicode** — C#'s `Main(string[] args)` receives the full UTF-16 strings from the OS
3. **Arguments are properly re-escaped** — following the [Microsoft C/C++ command-line parsing rules](https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments) before being passed to the target process

### Argument Escaping

Windows command-line argument passing has non-trivial rules for backslashes and double quotes:

- An argument containing spaces or quotes must be wrapped in double quotes
- Inside a quoted string, `\` before `"` must be doubled: `\\"`
- A trailing `\` before the closing `"` must also be doubled
- Backslashes not before a `"` remain as-is

UBP implements these rules correctly, unlike a naive `arg.Replace("\"", "\\\"")` which fails on paths ending with `\`.

### Long Path Support

The embedded [application manifest](../src/ubp.manifest) declares `longPathAware = true`, enabling paths longer than 260 characters on Windows 10 version 1607 and later (when the system policy is enabled).

### Error Handling

Since UBP runs as a GUI application, it can show error dialogs (via `MessageBoxW`) instead of silently failing:

- **No arguments:** Shows usage instructions
- **Target not found:** Shows which program couldn't be found
- **Launch failure:** Shows the exception message

## Data Flow

```
┌──────────────┐     ┌──────────┐     ┌─────────┐     ┌────────────────┐
│   Explorer   │────►│ Registry │────►│ ubp.exe │────►│ Target Program │
│ (right-click)│     │  (%V)    │     │ (winexe)│     │ (PS, Python..) │
└──────────────┘     └──────────┘     └─────────┘     └────────────────┘
                                           │
      Unicode path ────────────────────────┘
      preserved end-to-end
```

1. User right-clicks a file in Explorer
2. Explorer expands `%V` to the full Unicode path
3. Explorer calls `CreateProcessW()` with the registry command
4. UBP receives the arguments as Unicode strings in `Main(string[] args)`
5. UBP escapes the arguments correctly and calls `Process.Start()` on the target
6. The target program receives the intact Unicode path
