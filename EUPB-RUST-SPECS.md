# eupb — Specs v0.1 (réécriture Rust de Unicode Path Bridge)


Wrapper Windows minimaliste qui forwarde des arguments Unicode vers un
programme cible **sans fenêtre console**. Réécriture Rust du UPB C# original
(`C:\Users\janot\Claude\Unicode Path Bridge\`).

> **Statut** : projet sœur de `wslp`. Repo séparé prévu :
> `C:\Users\janot\Claude\eupb\` (à créer). Ces specs vivent temporairement dans
> le projet wslp ; à déplacer dans `eupb/SPECS.md` au démarrage du projet.

> Source de vérité du **quoi**. Le **comment** (jalons d'implémentation) sera
> dans `eupb/PLAN.md` une fois le projet créé.

- [ ] **Rename project** — Rename to "Explorer Unicode Path Bridge" / eupb.exe (repo, README, docs, code references, exe name). Update GitHub repo name accordingly.

## 1. Identité et portée

### 1.1 Mission

Un binaire unique `eupb.exe` qui résout 4 problèmes connus de l'intégration
shell Windows (clic droit Explorer → script/programme) :

| Problème | Cause | Effet utilisateur |
|---|---|---|
| **Corruption Unicode** | `powershell.exe` (PS 5.1) convertit les args via la code page ANSI locale | `C:\Тест\файл.txt` → `C:\???\????.txt` |
| **Format 8.3 short name** | `%1` en registre peut donner le short path | `C:\Dossier Été\` → `C:\DOSSIE~1\` |
| **Console flash** | Tout `.exe` console crée un conhost à chaque lancement | Fenêtre noire qui clignote ~50–200 ms |
| **Parsing `-Command`** | `powershell.exe -Command "& 'path'"` injecte le path dans le parser PS | Apostrophes (`L'été`) cassent le quoting |

Le binaire :
1. Est compilé en **`windows_subsystem = "windows"`** → aucune fenêtre console n'est créée.
2. Reçoit ses arguments en **UTF-16 préservé** (via `std::env::args_os` = `GetCommandLineW`).
3. Re-encode correctement les arguments (règles MS C/C++ command-line) avant de les passer à la cible.
4. Lance la cible via `CreateProcessW` avec les flags adéquats (`CREATE_NO_WINDOW` quand demandé).

### 1.2 Deux usages cibles

1. **Forwarder de path Unicode depuis Explorer** (usage historique EUPB) :
   ```
   "C:\Tools\eupb.exe" "powershell.exe" "-File" "C:\Scripts\foo.ps1" "%V"
   ```

2. **Wrapper "no-window" pour `wslp.exe` en clic droit** (cas d'usage qui motive cette réécriture) :
   ```
   "C:\Tools\eupb.exe" --hide-console "C:\Tools\wslp.exe" --clipboard --quiet -- "%V"
   ```
   Sans eupb, le clic droit sur "Copy WSL path" ferait flasher une console
   `wslp.exe`. Avec eupb en winexe, aucune fenêtre n'apparaît.

### 1.3 Non-objectifs

- Aucune logique métier (pas de conversion de chemin, pas de manipulation de fichier).
- Aucune édition de registre (eupb est **appelé depuis** le registre, il n'en écrit pas).
- Aucune TUI, aucun mode interactif.
- Aucune dépendance runtime externe (.NET, Visual C++ Redistributable, etc.).
- Pas de mode service/daemon.
- Pas d'auto-update interne (laissé à scoop/cargo/manuel).
- Pas de version Linux (Linux n'a pas le problème "fenêtre console flash" et n'a pas de menu contextuel Windows).

## 2. Choix technologiques

### 2.1 Langage & cible

- **Rust stable**, édition 2021.
- **Cible unique** : `x86_64-pc-windows-msvc` (Tier 1).
- Single-file, statiquement lié, pas de dépendance runtime.
- **Subsystem** : `windows` (pas de console). `#![windows_subsystem = "windows"]` au top du `main.rs`.

### 2.2 Compatibilité Windows

- **Minimum** : Windows 10 22H2 (build 19045) / Windows 11.
- Même rationale que wslp (cf. SPECS wslp §2.2) : Rust ≥ 1.78 a sorti Win7 du Tier 1.

### 2.3 Dépendances crates (prévisionnel)

| Besoin | Crate | Pourquoi |
|---|---|---|
| Parsing CLI | `clap` (derive) | `--help` auto, validation, cohérence avec wslp |
| Win32 (`CreateProcessW`, `MessageBoxW`) | `windows` (officiel MS) | Bindings officiels |
| Tests intégration | `assert_cmd`, `tempfile` | Standard Rust |
| Tests paramétrés | `rstest` | Tables de cas pour escaping |

Liste figée au M0 du projet eupb ; révisable par PR explicite.

### 2.4 Manifest applicatif Windows

Un `app.manifest` embarqué via `embed-resource` ou équivalent :

```xml
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
    </windowsSettings>
  </application>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/> <!-- Win10/11 -->
    </application>
  </compatibility>
</assembly>
```

Pas d'élévation UAC requise (`requestedExecutionLevel level="asInvoker"`).

## 3. CLI

### 3.1 Synopsis

```
eupb [OPTIONS] -- <TARGET> [TARGET_ARGS...]
eupb [OPTIONS] <TARGET> [TARGET_ARGS...]   (-- optionnel si pas d'ambiguïté)
```

Le séparateur `--` est **recommandé** dès que la target ou ses args peuvent
ressembler à des options eupb.

### 3.2 Options

| Option | Défaut | Effet |
|---|---|---|
| `--hide-console` | activé si target connu console | Lance la target avec `CREATE_NO_WINDOW` |
| `--show-console` | — | Force l'inverse (utile si target winexe — neutre) |
| `--wait` | activé | Attend la fin de la target, propage son exit code |
| `--no-wait` | — | Lance et exit 0 immédiatement (`DETACHED_PROCESS`) |
| `--cwd <DIR>` | dir actuel du parent | Définit le working directory de la target |
| `--show-errors` | activé | `MessageBoxW` sur échec de lancement (target introuvable, etc.) |
| `--quiet-errors` | — | Désactive les MessageBox ; juste exit code ≠ 0 |
| `--log <FILE>` | — | Log mode debug : args reçus, args ré-encodés, exit code, dans le fichier (UTF-8 + BOM pour Notepad) |
| `--version`, `-V` | — | Version |
| `--help`, `-h` | — | Aide (affichée via MessageBox car pas de console) |

**--hide-console` par défaut** : ,on assume console et on cache. L'utilisateur peut forcer avec
`--hide-console` / `--show-console`. Si la target ouvre quand même une fenêtre (parce qu'elle
est GUI et fait `MessageBox` ou crée une window), c'est son droit ; `CREATE_NO_WINDOW` ne supprime pas les fenêtres GUI, juste la console.

### 3.3 Comportement détaillé

1. **Lecture des arguments** :
   - Via `std::env::args_os()` qui appelle `GetCommandLineW` puis
     `CommandLineToArgvW` → préserve l'UTF-16 sans passer par la code page.
   - Première séparation : options eupb vs target+args (via `clap` qui supporte `--`).

2. **Validation** :
   - Si pas de target : `MessageBoxW` "Usage: eupb [options] target [args...]" et exit 1.
   - Si target n'existe pas (résolu via `where.exe` équivalent : recherche dans PATH si pas de slash, sinon path littéral) : `MessageBoxW` "Target not found: <target>" et exit 2.

3. **Re-encodage des args** :
   - Application des règles **Microsoft C/C++ command-line parsing** :
     - Argument sans space ni `"` → laissé tel quel.
     - Argument avec spaces ou tabs → wrappé dans `"..."`.
     - Backslashes avant un `"` → **doublés** dans le résultat.
     - Backslashes pas devant `"` → laissés tels quels.
     - Trailing `\` avant le closing `"` → **doublé**.
   - Ces règles sont implémentées dans une fonction `escape_arg(arg: &OsStr) -> OsString` testée à part.

4. **Lancement de la target** :
   - `CreateProcessW` avec :
     - `lpApplicationName = NULL`, `lpCommandLine = "<target> <escaped_args>"` (le mode "command line seule" gère bien l'escaping si on a fait notre boulot).
     - `dwCreationFlags = CREATE_NO_WINDOW` si `--hide-console`, sinon 0.
     - `dwCreationFlags |= DETACHED_PROCESS` si `--no-wait`.
     - `lpCurrentDirectory` selon `--cwd`.
     - `bInheritHandles = FALSE` (sécurité).
   - Si `--wait` : `WaitForSingleObject(hProcess, INFINITE)` puis `GetExitCodeProcess` → propage l'exit code.
   - Si `--no-wait` : ferme `hProcess` immédiatement, exit 0.

5. **Gestion d'erreurs** :
   - Tout échec d'API Windows → message lisible via `FormatMessageW` + code d'erreur, montré en MessageBox (ou silencieux si `--quiet-errors`).
   - Exit codes :
     - 0 : succès (ou exit code de la target en mode `--wait`)
     - 1 : usage error (pas de target)
     - 2 : target not found
     - 3 : `CreateProcessW` échec
     - 4 : `WaitForSingleObject` / `GetExitCodeProcess` échec
     - >100 : exit code de la target propagé tel quel

### 3.4 Encodages

- I/O console : non applicable (binaire winexe, pas de console).
- Arguments : `WCHAR` bout-en-bout (Win32 natif).
- Fichier de log (`--log`) : UTF-8 avec BOM (pour ouverture immédiate dans Notepad sans bug d'encodage).
- MessageBox : `MessageBoxW` (Unicode natif).

## 4. Sécurité

### 4.1 Surface d'attaque

eupb lance des processus arbitraires avec des arguments arbitraires.
Restrictions :

- **Pas d'élévation** : `requestedExecutionLevel="asInvoker"`. eupb ne peut pas privilèges.
- **`bInheritHandles = FALSE`** : la target n'hérite pas des handles (stdin/stdout/stderr) du parent. Pas de fuite.
- **Validation minimale** : on vérifie l'existence de la target, mais pas sa signature, pas son contenu. eupb est un transport, pas un gardien.
- **Pas de variables d'environnement injectées** : la target hérite de l'environnement courant tel quel.

### 4.2 Quoting et injection

Le risque principal d'un wrapper qui construit une `lpCommandLine` est
l'injection d'arguments via un argument malicieusement formé. La fonction
`escape_arg` doit être :

- **Couverte de tests** avec une table de >50 cas (cf. §6.1).
- **Auditée manuellement** sur les cas tordus : `"`, `\\`, `\\"`, `"\\"`, args vides, args avec uniquement des espaces, etc.
- **Non modifiée à la légère** : tout changement de `escape_arg` doit avoir une justification écrite et passer les tests existants + nouveaux cas.

## 5. Build & distribution

### 5.1 Build

- `cargo build --release --target x86_64-pc-windows-msvc`
- Profil release : `lto = true`, `strip = true`, `codegen-units = 1`, `panic = "abort"` → binaire ~300–500 KB attendu.
- Embed du manifest via `embed-resource` dans `build.rs`.
- Embed d'une icône (optionnel, à fournir).

### 5.2 Distribution

| Canal | Forme |
|---|---|
| GitHub releases | Zip avec `eupb.exe` + `README.md` + `LICENSE` |
| crates.io | `cargo install eupb-bridge` (nom à finaliser, "eupb" est sans doute pris) |
| Scoop bucket | `bucket/eupb.json` minimal (pas de post-install, juste copie en bin) |
| Bundle dans wslp | Wslp release zip contient `eupb.exe` à côté de `wslp.exe` |

Pas d'installeur dédié. eupb est tellement simple qu'une copie manuelle ou
`cargo install` suffit.

### 5.3 Versions

- **v0.1.0** : feature parity avec UPB C# original + `--hide-console` + `--no-wait` + `--log`.
- **v0.2.0+** : à définir selon retours d'usage (heuristiques, file-association tools, etc.).

## 6. Tests

### 6.1 Tests unitaires — escaping

Cas couverts dans `tests/escape.rs` (table-driven via `rstest`) :

- Argument vide
- Argument simple sans special chars : `foo`
- Avec spaces : `hello world` → `"hello world"`
- Avec quotes : `say "hi"` → `"say \"hi\""`
- Avec backslashes seuls : `C:\Users` → `C:\Users` (pas de wrap si pas de space)
- Avec backslashes ET spaces : `C:\Program Files` → `"C:\Program Files"`
- Trailing backslash sans space : `C:\Users\` → `C:\Users\`
- Trailing backslash avec space : `C:\My Stuff\` → `"C:\My Stuff\\"` (doublé)
- Backslashes avant quote : `\\"` → `\\\\\"` (chacun doublé + quote escapée)
- Args avec accents : `café` → `café`
- Args CJK : `日本` → `日本`
- Args emoji : `🎉` → `🎉`
- Apostrophes : `L'été` → `L'été` (pas un caractère spécial pour cmd line Windows)
- Args très longs (>1 KB)
- Path avec long path prefix : `\\?\C:\very\long\path\...`

### 6.2 Tests intégration

Binaire de test `eupb-test-target.exe` (compilé dans le projet) :
- Reçoit ses args, les sérialise en JSON UTF-8 sur un fichier passé en `--out`.
- Le test eupb lance `eupb.exe eupb-test-target.exe --out result.json <args_à_tester>`, lit `result.json`, compare au attendu.

Tests round-trip :
- Args ASCII simples
- Args Unicode (5+ scripts : latin-accentué, cyrillique, CJK, hébreu, emoji)
- Args avec quotes/backslashes/spaces
- Args > 260 chars
- Args avec long path prefix

### 6.3 Tests manuels

Documentés dans `TEST-MANUAL.md` du projet eupb :

- Import d'un `.reg` qui déclare un menu contextuel utilisant `eupb.exe`.
- Clic droit sur 5+ fichiers Unicode dans Explorer → vérifie absence de flash + bonne réception côté target (un script PS de test qui dump ses args dans un fichier temp).
- Test depuis cmd.exe : `eupb.exe --hide-console notepad.exe` → notepad s'ouvre, pas de console résiduelle.
- Test du `--no-wait` : `eupb.exe --no-wait notepad.exe` → eupb exit immédiatement, notepad reste ouvert.
- Test du `--log` : vérifie que le fichier de log est lisible dans Notepad sans souci d'encodage.

### 6.4 Discipline de tests

- Pas de test qui modifie le registre réel de la machine de dev.
- Tests intégration utilisent `tempfile::TempDir` pour les fichiers de sortie.
- Aucun test ne touche au menu contextuel réel (réservé à `TEST-MANUAL.md`).

## 7. Backlog (post v0.1)

- Heuristique de détection GUI vs console plus fine.
- Mode "file association" (s'enregistrer comme handler par défaut pour des extensions).
- Variant `eupb-cli.exe` console (pour debug, mais en pratique `--log` suffit).
- Support ARM64 Windows.
- Mode "lancement délayé" (`--delay <ms>`) pour debug.


- [ ] **Output encoding switch** — Add a CLI parameter (e.g., `--encoding utf8`) to control how the path is encoded when passed to the target program (UTF-8, UTF-16, ANSI, etc.). Useful for targets that expect a specific encoding.

- [ ] **Install script** — Interactive PowerShell script that registers a context menu entry in the Windows registry. Should prompt for:
  - Target program path
  - Additional arguments (optional)
  - Menu entry display name
  - Icon (file path or shell32.dll index)
  - Registry scope (multi-select):
    - `HKCR\*\shell` (files only)
    - `HKCR\Directory\shell` (folders only)
    - `HKCR\AllFilesystemObjects\shell` (files and folders)
    - `HKCR\Directory\Background\shell` (folder background)

- [ ] **Uninstall script** — Companion to the install script. Should:
  - List all eupb-registered context menu entries
  - Allow selective or full removal
  - Clean up registry keys created by the install script