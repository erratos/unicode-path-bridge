# TODO — eupb

Not part of the published project (gitignored). Working notes.

## v0.2+

- **Capture target stdout to a file** — `--stdout-file FILE` (and
  `--stderr-file FILE`), writing UTF-8 with BOM. Useful when the target
  produces text that's easier to post-process from a file than from a
  pipe (and `eupb` is already the only thing the `.reg` entry gets to
  invoke, so no shell redirection is available).

- **Capture target stdout to the clipboard** — `--stdout-clipboard`,
  reading the child's stdout, stripping a trailing newline, pushing
  UTF-16 to the Windows clipboard via `OpenClipboard` +
  `SetClipboardData(CF_UNICODETEXT, ...)`. Makes "Run a script and copy
  its output" a single-line registry entry. Pairs well with
  `--set-env` for input.

- **Feed stdin to the target** — `--stdin-string STR` or
  `--stdin-file FILE`, to pass Unicode payloads without going through
  a shell. Natural counterpart to `--stdout-*`.

- **Timeout** — `--timeout MS`, kill the child if it runs too long
  (use `TerminateProcess` after `WaitForSingleObject` returns
  `WAIT_TIMEOUT`). Guards against targets that hang silently under
  `--hide-console`.

- **Clear / minimal environment** — `--clear-env` to start the child
  with only the `--set-env` pairs (plus the bare minimum Windows needs
  like `SystemRoot`). Useful for reproducible invocations and for
  isolating scripts from the user's env.

- **Notification on completion** — `--notify-on-exit` or
  `--notify-on-error`, pop a toast (or a `MessageBox`) when a detached
  or long-running target finishes. Closes the UX gap caused by
  `--hide-console` swallowing all feedback.

- **Output encoding switch** — `--encoding utf8|utf16|ansi` to control
  how the forwarded arguments are re-encoded for targets that expect a
  specific encoding (most targets are fine with UTF-16 from
  `CreateProcessW`, but edge cases exist).

- **Install script** — Interactive PowerShell script that registers a
  context menu entry in the Windows registry. Prompts for:
  - Target program path
  - Additional arguments (optional)
  - Menu entry display name
  - Icon (file path or `shell32.dll` index)
  - Registry scope (multi-select):
    - `HKCR\*\shell` (files only)
    - `HKCR\Directory\shell` (folders only)
    - `HKCR\AllFilesystemObjects\shell` (files and folders)
    - `HKCR\Directory\Background\shell` (folder background)

- **Uninstall script** — Companion to the install script. Should:
  - List all eupb-registered context menu entries
  - Allow selective or full removal
  - Clean up registry keys created by the install script

## Post v0.2

- Heuristic detection of GUI vs console target (so `--hide-console`
  becomes a no-op automatically for GUI targets).
- ARM64 Windows build.
- `crates.io` publication (name is likely to be `eupb-bridge` or
  similar — check availability).
- Scoop manifest.

## Pre-release checklist (once a version is ready to tag)

- [ ] `cargo test` green on a fresh clone
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` applied
- [ ] `cargo build --release` produces a single `eupb.exe` < 1 MB
- [ ] Manual: right-click on a file with a Cyrillic + emoji path, run
      through a `.reg` example, confirm no console flash and correct
      path reception
- [ ] CHANGELOG updated
- [ ] README version bumped in examples if relevant
- [ ] Git tag + GitHub release with `eupb.exe` attached
