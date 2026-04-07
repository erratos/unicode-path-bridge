# Registry Setup Guide

This guide explains how to add custom right-click context menu entries that use Unicode Path Bridge.

## Registry Basics

Windows Explorer's context menu is driven by keys under `HKEY_CLASSES_ROOT` (HKCR). The structure is:

```
HKCR\<scope>\shell\<entry-name>\
    (Default) = "Menu Text"
    "Icon" = "path-to-icon"
HKCR\<scope>\shell\<entry-name>\command\
    (Default) = "command to execute"
```

## Choosing the Right Scope

| Registry path | Applies to |
|--------------|------------|
| `HKCR\*\shell\` | Files only |
| `HKCR\Directory\shell\` | Folders only |
| `HKCR\AllFilesystemObjects\shell\` | **Both files and folders** (recommended) |
| `HKCR\Directory\Background\shell\` | Folder background (right-click empty space) |

For most use cases, `AllFilesystemObjects` is the best choice.

## The %V Token

**Always use `%V` instead of `%1` in your commands.**

| Token | Behavior |
|-------|----------|
| `%1` | Legacy MS-DOS token. May return the 8.3 short path (`C:\DOSSIE~1\FILE.TXT`) |
| `%V` | Forces the full long path (`C:\Dossier Été\Fichier.txt`) |
| `%L` | Same as `%V` for most purposes |

## Command Format

The command value must follow this pattern:

```
"C:\path\to\ubp.exe" "target-program" "arg1" "arg2" "%V"
```

Important rules:
- **Double all backslashes** in `.reg` files (`\\` instead of `\`)
- **Wrap each argument in escaped quotes** (`\"...\"`), especially paths with spaces
- `%V` should be the last argument (it's the path Explorer fills in)

## Step-by-Step: Adding a Context Menu Entry

### Option A: Using a .reg File

1. Open Notepad (or any text editor)
2. Paste this template:

```reg
Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\AllFilesystemObjects\shell\MyCustomAction]
@="My Custom Action"
"Icon"="shell32.dll,1"

[HKEY_CLASSES_ROOT\AllFilesystemObjects\shell\MyCustomAction\command]
@="\"C:\\Tools\\ubp.exe\" \"powershell.exe\" \"-ExecutionPolicy\" \"Bypass\" \"-File\" \"C:\\Scripts\\my-script.ps1\" \"%V\""
```

3. Save as `my-action.reg` (make sure the encoding is UTF-8 or ANSI)
4. Double-click the `.reg` file and confirm the import
5. Right-click any file or folder — you should see "My Custom Action"

### Option B: Using the Registry Editor

1. Open `regedit.exe` (Run as Administrator)
2. Navigate to `HKEY_CLASSES_ROOT\AllFilesystemObjects\shell`
3. Create a new key with your action name (e.g., `MyCustomAction`)
4. Set its `(Default)` value to the menu text
5. Optionally add an `Icon` string value (e.g., `shell32.dll,134` for a clipboard icon)
6. Create a `command` subkey under it
7. Set the `command` key's `(Default)` value to the command string (with single backslashes this time — the doubling is only needed in `.reg` files)

### Option C: Using PowerShell

```powershell
# Run as Administrator
$keyPath = "Registry::HKEY_CLASSES_ROOT\AllFilesystemObjects\shell\MyCustomAction"
New-Item -Path $keyPath -Force
Set-ItemProperty -Path $keyPath -Name "(Default)" -Value "My Custom Action"
Set-ItemProperty -Path $keyPath -Name "Icon" -Value "shell32.dll,1"

New-Item -Path "$keyPath\command" -Force
Set-ItemProperty -Path "$keyPath\command" -Name "(Default)" -Value '"C:\Tools\ubp.exe" "powershell.exe" "-ExecutionPolicy" "Bypass" "-File" "C:\Scripts\my-script.ps1" "%V"'
```

## Common Icons

You can use any `.ico` file, or reference built-in Windows icons:

| Value | Icon |
|-------|------|
| `shell32.dll,1` | Document |
| `shell32.dll,3` | Folder (closed) |
| `shell32.dll,4` | Folder (open) |
| `shell32.dll,134` | Clipboard |
| `shell32.dll,144` | Blue folder |
| `imageres.dll,2` | Film strip |
| `imageres.dll,11` | Paint palette |

## Removing an Entry

### Using a .reg File

Prefix the key path with a minus sign:

```reg
Windows Registry Editor Version 5.00

[-HKEY_CLASSES_ROOT\AllFilesystemObjects\shell\MyCustomAction]
```

See [`examples/uninstall.reg`](../examples/uninstall.reg) for a ready-made cleanup file.

### Using the Registry Editor

Navigate to the key in `regedit.exe`, right-click it, and select "Delete".

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Menu entry doesn't appear | Wrong registry path or typo | Verify the key exists in `regedit` |
| "Target program not found" error | Wrong path to the target program | Check the path and make sure backslashes are correct |
| Path has `?` characters | Not using UBP, or using `-Command` instead of `-File` | Make sure the command goes through `ubp.exe` |
| Path is in 8.3 format | Using `%1` instead of `%V` | Replace `%1` with `%V` in the command |
| Console window flashes | Not using UBP, or UBP not compiled as `winexe` | Rebuild with `/target:winexe` |
