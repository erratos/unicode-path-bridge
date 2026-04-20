# Handoff — eupb (Explorer Unicode Path Bridge)

**Statut** : v0.1.0 complet, test suite 100%, prêt pour les prochaines étapes.

## Derniers commits (session 2026-04-20)

1. **880e433** — fix: don't canonicalize target path by default (fixes Electron/VSCodium)
   - Suppression de `std::fs::canonicalize()` par défaut (ajoutait `\?\` prefix incompatible avec Electron)
   - Ajout flag `--canonicalize` pour symlinks (opt-in)
   
2. **4d3be16** — feat: make --no-wait the default, add --wait-exit for opt-in waiting
   - `--no-wait` est le nouveau défaut (mieux pour Explorer)
   - `--wait-exit` pour opt-in quand on veut attendre le processus enfant
   
3. **0699163** — chore: registry entries cosmetic fixes
   - Nettoyage exemples `.reg`

## Prochaines étapes

Ordre suggéré :

1. **Validation manuelle clic droit étendue** 
   - `copy-path.reg` validé sur 4 chemins (CJK, accents, emoji, apostrophe + espace)
   - À tester : `run-script.reg`, `open-in-vscode.reg` sur dossiers cyrillique/CJK/emoji
   - Vérifier : pas de flash console, chemin intact côté script cible

2. **Tag v0.1.0 + GitHub release**
   - Ajouter binaire `eupb.exe` (309 KB) en attachment
   - Binaire dans `target/release/` et copié à `C:\Tools\eupb.exe`

3. **`cargo clippy -- -D warnings` et `cargo fmt`**
   - Pas encore passés, lints possibles à nettoyer

4. **Backlog** (TODO.md) — "bridge adaptatif"
   - `--stdout-file`, `--stdout-clipboard`
   - `--stdin-string`, `--stdin-file`
   - `--timeout`, `--clear-env`, `--notify-on-exit`
   - `--encoding` switch
   - install/uninstall interactifs

5. **Publication crates.io** (optionnel)
   - Nom `eupb` probablement disponible

## Contexte technique

| Aspect | Détails |
|---|---|
| **Cible** | `x86_64-pc-windows-msvc`, Rust 1.78+ |
| **Subsystem** | release=`windows` (pas de console), debug=default |
| **UTF-16** | Bout-en-bout : `args_os()` → `escape_arg_wide` → `CreateProcessW` |
| **Escape rules** | Littéral MS C/C++ spec : doubles backslashes avant `"` ou trailing |
| **Exit codes** | 0=succès/code propagé, 1=usage, 2=target not found, 3=CreateProcessW KO, 4=wait KO |
| **Manifest** | `longPathAware`, `activeCodePage=UTF-8`, asInvoker, Win10/11 |
| **Release binary** | 309 KB (lto, strip, panic=abort) |
| **Tests** | 52 intégration (ASCII/Unicode, parité .NET, `--set-env`) + 29 unitaires escape |

## Fichiers clés

| Fichier | Rôle |
|---|---|
| [Cargo.toml](Cargo.toml) | Deps, profile release |
| [src/lib.rs](src/lib.rs) | `escape_arg` API publique |
| [src/main.rs](src/main.rs) | CLI + glue + resolve_executable |
| [src/win.rs](src/win.rs) | `CreateProcessW`, env block, errors |
| [tests/integration.rs](tests/integration.rs) | 52 tests (Unicode, parité, env vars) |
| [examples/*.reg](examples/) | 7 templates clic droit |
| [EUPB-RUST-SPECS.md](EUPB-RUST-SPECS.md) | Specs source de vérité (gitignored) |
| [TODO.md](TODO.md) | Backlog post-v0.1 (gitignored) |
| `.archive/dotnet/` | Ancien code C# (gitignored) |

## Quick reference

```bash
# Test complet
cargo test

# Build release
cargo build --release

# Deploy
cp target/release/eupb.exe C:\Tools\eupb.exe

# Import registry
reg import examples/copy-path.reg
```

**Branch** : `main` (tout pushé à origin)
