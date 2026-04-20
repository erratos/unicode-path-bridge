//! eupb — Explorer Unicode Path Bridge
//!
//! A no-console wrapper that forwards Unicode arguments from the Windows
//! Explorer context menu (or any caller) to a target program, preserving
//! the arguments end-to-end via UTF-16 and proper command-line escaping.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser};

#[cfg(windows)]
mod win;

#[derive(Parser, Debug)]
#[command(
    name = "eupb",
    version,
    about = "Explorer Unicode Path Bridge — forward Unicode args to a target without a console flash",
    disable_help_flag = true,
    disable_version_flag = true,
    trailing_var_arg = true,
    allow_hyphen_values = true
)]
struct Cli {
    /// Hide the target's console window (CREATE_NO_WINDOW). Default.
    #[arg(long, overrides_with = "show_console")]
    hide_console: bool,

    /// Keep the target's console window visible.
    #[arg(long, overrides_with = "hide_console")]
    show_console: bool,

    /// Wait for the target to exit and propagate its exit code. Default.
    #[arg(long, overrides_with = "no_wait")]
    wait: bool,

    /// Do not wait; launch detached and exit 0 immediately.
    #[arg(long, overrides_with = "wait")]
    no_wait: bool,

    /// Working directory for the target.
    #[arg(long, value_name = "DIR")]
    cwd: Option<PathBuf>,

    /// Show error dialogs on launch failure. Default.
    #[arg(long, overrides_with = "quiet_errors")]
    show_errors: bool,

    /// Suppress error dialogs; rely on exit codes only.
    #[arg(long, overrides_with = "show_errors")]
    quiet_errors: bool,

    /// Log invocation details to this file (UTF-8 with BOM).
    #[arg(long, value_name = "FILE")]
    log: Option<PathBuf>,

    /// Canonicalize the target path (resolve symlinks, normalize dots).
    /// On Windows this adds a `\\?\` extended-length prefix which may break
    /// Electron apps (VS Code, VSCodium, etc.). Disabled by default.
    #[arg(long)]
    canonicalize: bool,

    /// Set or override an environment variable for the target process.
    /// Format: `NAME=VALUE`. Repeatable. Useful to pass Unicode paths to
    /// shells like PowerShell whose `-Command` parser would otherwise
    /// reinterpret characters like `&`, `'`, `;` or `$`.
    #[arg(long = "set-env", value_name = "NAME=VALUE", action = clap::ArgAction::Append)]
    set_env: Vec<OsString>,

    /// Print help.
    #[arg(long, short = 'h')]
    help: bool,

    /// Print version.
    #[arg(long, short = 'V')]
    version: bool,

    /// Target program followed by its arguments. `--` separates eupb options
    /// from target arguments cleanly; it is optional if unambiguous.
    #[arg(required = false, value_name = "TARGET_AND_ARGS")]
    target_args: Vec<OsString>,
}

struct Resolved {
    target: PathBuf,
    args: Vec<OsString>,
    hide_console: bool,
    wait: bool,
    cwd: Option<PathBuf>,
    show_errors: bool,
    log: Option<PathBuf>,
    set_env: Vec<(OsString, OsString)>,
}

fn main() -> ExitCode {
    // Peek for --quiet-errors before parsing so we can suppress dialogs
    // even when clap itself rejects the argument set.
    let raw_args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let quiet_early = raw_args
        .iter()
        .any(|a| a == "--quiet-errors");

    let parsed = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            if !quiet_early {
                show_dialog_info("eupb — argument error", &e.to_string());
            }
            return ExitCode::from(1);
        }
    };

    if parsed.help {
        let help = Cli::command().render_long_help().to_string();
        show_dialog_info("eupb — help", &help);
        return ExitCode::from(0);
    }
    if parsed.version {
        show_dialog_info(
            "eupb — version",
            &format!("eupb {}", env!("CARGO_PKG_VERSION")),
        );
        return ExitCode::from(0);
    }

    // Resolve defaults (hide_console default = true, wait default = true, show_errors default = true)
    let hide_console = !parsed.show_console;
    let wait = !parsed.no_wait;
    let show_errors = !parsed.quiet_errors;

    // Require at least a target.
    if parsed.target_args.is_empty() {
        if show_errors {
            show_dialog_error(
                "eupb — usage",
                "Usage: eupb [OPTIONS] -- <TARGET> [TARGET_ARGS...]\n\nNo target program specified.",
            );
        }
        return ExitCode::from(1);
    }

    let target_name = &parsed.target_args[0];
    let target = match resolve_executable(target_name, parsed.canonicalize) {
        Some(p) => p,
        None => {
            if show_errors {
                show_dialog_error(
                    "eupb — target not found",
                    &format!("Target program not found: {}", target_name.to_string_lossy()),
                );
            }
            return ExitCode::from(2);
        }
    };

    let args: Vec<OsString> = parsed.target_args.iter().skip(1).cloned().collect();

    let set_env = match parse_set_env_pairs(&parsed.set_env) {
        Ok(v) => v,
        Err(msg) => {
            if show_errors {
                show_dialog_error("eupb — usage", &msg);
            }
            return ExitCode::from(1);
        }
    };

    let resolved = Resolved {
        target,
        args,
        hide_console,
        wait,
        cwd: parsed.cwd,
        show_errors,
        log: parsed.log,
        set_env,
    };

    #[cfg(windows)]
    {
        win::launch(&resolved)
    }

    #[cfg(not(windows))]
    {
        let _ = &resolved;
        eprintln!("eupb only runs on Windows.");
        ExitCode::from(1)
    }
}

/// Parse each `--set-env` argument as `NAME=VALUE`. Splits on the **first**
/// `=` (in UTF-16 on Windows, in bytes elsewhere — only the ASCII `=` is
/// significant, so either produces identical results for any real input).
/// Rejects missing `=` and empty `NAME`.
#[cfg(windows)]
fn parse_set_env_pairs(raws: &[OsString]) -> Result<Vec<(OsString, OsString)>, String> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    let eq = b'=' as u16;
    let mut out = Vec::with_capacity(raws.len());
    for raw in raws {
        let wide: Vec<u16> = raw.encode_wide().collect();
        let Some(pos) = wide.iter().position(|&c| c == eq) else {
            return Err(format!(
                "--set-env requires NAME=VALUE (got: {})",
                raw.to_string_lossy()
            ));
        };
        if pos == 0 {
            return Err(format!(
                "--set-env NAME cannot be empty (got: {})",
                raw.to_string_lossy()
            ));
        }
        let name = OsString::from_wide(&wide[..pos]);
        let value = OsString::from_wide(&wide[pos + 1..]);
        out.push((name, value));
    }
    Ok(out)
}

#[cfg(not(windows))]
fn parse_set_env_pairs(_raws: &[OsString]) -> Result<Vec<(OsString, OsString)>, String> {
    Ok(Vec::new())
}

/// Resolve a program name to an absolute path. If `name` contains a path
/// separator, treat it as a literal path. Otherwise search PATH with PATHEXT.
///
/// `canonicalize`: if true, call `std::fs::canonicalize` (resolves symlinks and
/// normalizes dots, but adds a `\\?\` extended-length prefix on Windows which
/// breaks Electron apps). Default is false — the path is used as-is.
fn resolve_executable(name: &OsStr, canonicalize: bool) -> Option<PathBuf> {
    let as_path = Path::new(name);
    let name_str = name.to_string_lossy();

    if name_str.contains('/') || name_str.contains('\\') {
        return if as_path.is_file() {
            if canonicalize {
                std::fs::canonicalize(as_path).ok().or_else(|| Some(as_path.to_path_buf()))
            } else {
                Some(as_path.to_path_buf())
            }
        } else {
            None
        };
    }

    let has_ext = as_path.extension().is_some();
    let pathext_os = std::env::var_os("PATHEXT")
        .unwrap_or_else(|| OsString::from(".EXE;.CMD;.BAT;.COM"));
    let pathext = pathext_os.to_string_lossy();
    let exts: Vec<&str> = pathext.split(';').filter(|s| !s.is_empty()).collect();

    let path_env = std::env::var_os("PATH")?;
    let path_str = path_env.to_string_lossy();

    for dir in path_str.split(';').filter(|s| !s.is_empty()) {
        let d = Path::new(dir);
        if has_ext {
            let c = d.join(name);
            if c.is_file() {
                return Some(c);
            }
        } else {
            for ext in &exts {
                let mut candidate = name_str.to_string();
                candidate.push_str(ext);
                let c = d.join(candidate);
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn show_dialog_error(caption: &str, text: &str) {
    win::message_box(caption, text, true);
}

#[cfg(windows)]
fn show_dialog_info(caption: &str, text: &str) {
    win::message_box(caption, text, false);
}

#[cfg(not(windows))]
fn show_dialog_error(caption: &str, text: &str) {
    eprintln!("[{}] {}", caption, text);
}

#[cfg(not(windows))]
fn show_dialog_info(caption: &str, text: &str) {
    eprintln!("[{}] {}", caption, text);
}
