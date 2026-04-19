# Alternatives to Unicode Path Bridge

UBP is not the only way to solve the Unicode context menu problem. Here's an honest comparison to help you decide which approach fits your situation.

## PowerShell 7 (`pwsh.exe`)

**What it does:** PowerShell 7 natively handles Unicode arguments correctly.

**When to use it instead of UBP:**
- You (and your users) already have PowerShell 7 installed
- You only need to run PowerShell scripts (not Python or other programs)
- You're OK with the ~400 MB install footprint

**When UBP is better:**
- Your target machines only have the default Windows PowerShell 5.1
- You need to forward paths to non-PowerShell programs (Python, custom tools)
- You want zero additional dependencies

**Registry example with pwsh.exe:**
```
"C:\Program Files\PowerShell\7\pwsh.exe" -ExecutionPolicy Bypass -File "C:\Scripts\my-script.ps1" "%V"
```

Note: You'll still get a console flash unless you also use a GUI wrapper.

## cubiclesoft/createprocess-windows

**Repository:** https://github.com/cubiclesoft/createprocess-windows

**What it does:** A comprehensive `CreateProcess()` wrapper with dozens of options (priority, affinity, environment, window position, etc.).

**When to use it instead of UBP:**
- You need fine-grained control over how the child process is created
- You need features like process ID output, custom environment, or window positioning

**When UBP is better:**
- You want something simple — UBP has no options to learn
- You want to compile from source easily (C# with built-in compiler vs. C++ requiring a toolchain)

## SharpShell / COM Shell Extensions

**Repository:** https://github.com/dwmkerr/sharpshell

**What it does:** A .NET framework for building proper COM shell extensions (context menus, icon overlays, etc.).

**When to use it instead of UBP:**
- You're building a polished, distributable application
- You need sub-menus, dynamic menu items, or icons per file type
- You want the "proper" way to extend Explorer

**When UBP is better:**
- You just want to forward a path to a script
- You don't want to deal with COM registration, GAC, or administrator approval
- You want a solution that's a single file, not a framework

## VBScript (`wscript.exe`)

**What it does:** Launches programs without a visible window (since `wscript.exe` is a GUI-subsystem process).

**Example:**
```vbs
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "powershell.exe -File ""C:\Scripts\script.ps1"" """ & WScript.Arguments(0) & """", 0, False
```

**When to use it instead of UBP:** Honestly, never. VBScript:
- Processes strings through the ANSI code page, reintroducing Unicode corruption
- Is officially deprecated by Microsoft
- Has significant security concerns (frequently used as a malware vector)

UBP was specifically created to replace this approach.

## Summary

| Feature | UBP | PS7 | createprocess-windows | SharpShell | VBScript |
|---------|-----|-----|-----------------------|------------|----------|
| Unicode paths | Yes | Yes | Yes | Yes | **No** |
| No console flash | Yes | No | Yes (`-win` variant) | Yes | Yes |
| Zero dependencies | Yes | No (install required) | Yes | No (.NET, COM) | Yes |
| Easy to compile | Yes (built-in csc) | N/A | No (C++ toolchain) | No (Visual Studio) | N/A (interpreted) |
| Single file solution | Yes | N/A | Yes | No | Yes |
| Non-PS targets | Yes | No | Yes | Yes | Yes |
| Actively maintained | Yes | Yes | Yes | Varies | **Deprecated** |
