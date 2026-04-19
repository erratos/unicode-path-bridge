//! Test helper binary used by the integration test suite.
//!
//! Writes its received argv (everything after `--out <file>`) to the given
//! file as UTF-8 JSON, then exits 0.
//!
//! Build as a plain console binary — the integration tests launch it through
//! eupb.exe, which wraps it under CREATE_NO_WINDOW, so no console is shown.

use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();

    // Find --out <path> anywhere; everything else is captured.
    // --env-key NAME appends "NAME=<value>" (or "NAME=<UNSET>") after the regular args.
    let mut out_path: Option<PathBuf> = None;
    let mut captured: Vec<String> = Vec::new();
    let mut env_keys: Vec<OsString> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--out" {
            if i + 1 >= args.len() {
                eprintln!("--out requires a value");
                return ExitCode::from(2);
            }
            out_path = Some(PathBuf::from(&args[i + 1]));
            i += 2;
            continue;
        }
        if a == "--env-key" {
            if i + 1 >= args.len() {
                eprintln!("--env-key requires a value");
                return ExitCode::from(2);
            }
            env_keys.push(args[i + 1].clone());
            i += 2;
            continue;
        }
        captured.push(a.to_string_lossy().into_owned());
        i += 1;
    }

    for k in &env_keys {
        let value = std::env::var_os(k);
        let key_str = k.to_string_lossy();
        match value {
            Some(v) => captured.push(format!("{}={}", key_str, v.to_string_lossy())),
            None => captured.push(format!("{}=<UNSET>", key_str)),
        }
    }

    let out = match out_path {
        Some(p) => p,
        None => {
            eprintln!("missing --out <path>");
            return ExitCode::from(2);
        }
    };

    let mut items = String::from("[");
    for (idx, s) in captured.iter().enumerate() {
        if idx > 0 {
            items.push(',');
        }
        items.push_str(&json_string(s));
    }
    items.push(']');

    let mut f = match File::create(&out) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to create {}: {}", out.display(), e);
            return ExitCode::from(3);
        }
    };
    if let Err(e) = f.write_all(items.as_bytes()) {
        eprintln!("failed to write {}: {}", out.display(), e);
        return ExitCode::from(3);
    }
    ExitCode::from(0)
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
