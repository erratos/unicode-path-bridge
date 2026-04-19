# Registry setup

A minimal guide to adding an Explorer right-click entry that runs
through `eupb.exe`. For a broader tour of the Windows shell registry
(differences between `HKCR\*\shell`, `HKCR\Directory\shell`, etc.), see
[`archive/dotnet/docs/registry-setup.md`](../archive/dotnet/docs/registry-setup.md)
— that content was written for the C# version but the registry concepts
are identical.

## 1. Install eupb

1. Build with `cargo build --release` (or download a prebuilt
   `eupb.exe`).
2. Copy `target\release\eupb.exe` to a stable location, e.g.
   `C:\Tools\eupb.exe`.

## 2. Pick a scope

| Key | Shows up on |
|---|---|
| `HKCR\*\shell\<Name>` | All files |
| `HKCR\Directory\shell\<Name>` | Folders |
| `HKCR\AllFilesystemObjects\shell\<Name>` | Files and folders |
| `HKCR\Directory\Background\shell\<Name>` | Folder *background* (right-click inside a folder) |

## 3. Write the command

Always use `%V` (long Unicode path), not `%1` (may be 8.3 short name):

```
"C:\Tools\eupb.exe" -- "powershell.exe" "-NoProfile" "-File" "C:\Scripts\my-script.ps1" "%V"
```

Key points:

- The `--` separator prevents eupb from interpreting target arguments
  as its own options.
- Every path argument is double-quoted because the registry passes the
  command line as a single string — Explorer only substitutes `%V`, it
  does not add quotes.
- `eupb.exe` re-escapes each argument correctly before handing them to
  `CreateProcessW`, so quotes and backslashes inside `%V` survive.

## 4. Example .reg

See [`examples/run-script.reg`](../examples/run-script.reg),
[`examples/copy-path.reg`](../examples/copy-path.reg), and
[`examples/open-in-vscode.reg`](../examples/open-in-vscode.reg) for
ready-to-import snippets.

## 5. Uninstall

Delete the key you created. See [`examples/uninstall.reg`](../examples/uninstall.reg)
for an example that removes the entries shipped here.

## Common pitfalls

| Symptom | Cause | Fix |
|---|---|---|
| `?` characters instead of Cyrillic / CJK | Target program (often PowerShell 5.1) converts via ANSI | Already the reason eupb exists — make sure the target is actually being launched through `eupb.exe` |
| `DOSSIE~1` instead of `Dossier Été` | Used `%1` in the registry command | Replace with `%V` |
| Brief black console flash | Omitted `--hide-console` or using `--show-console` | Remove `--show-console`; the default hides the window |
| Explorer blocks until the script finishes | `eupb` waits by default | Add `--no-wait` to the command line |
