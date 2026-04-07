# Changelog

## v1.0.0 — 2026-04-07

Initial release.

- Unicode-safe argument forwarding from Explorer context menu to any target program
- No console window flash (compiled as Windows application)
- Proper Windows command-line argument escaping (handles spaces, quotes, backslashes)
- Long path support (>260 characters) via application manifest
- Error dialogs for missing target program or launch failure
- PATH resolution for target programs (e.g., `powershell.exe` without full path)
- Example `.reg` files for common use cases
- Automated test suite covering French accents, Cyrillic, CJK, emoji, and edge cases
