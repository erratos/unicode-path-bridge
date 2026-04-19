//! Integration tests: spawn eupb.exe → eupb-test-target.exe and verify
//! that arguments survive the round-trip unchanged.

#![cfg(windows)]

use assert_cmd::cargo::CommandCargoExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn target_path() -> PathBuf {
    // eupb-test-target shares the target/<profile>/ directory with eupb.
    let eupb = Command::cargo_bin("eupb")
        .expect("build eupb")
        .get_program()
        .to_os_string();
    let mut p = PathBuf::from(eupb);
    p.pop();
    p.push("eupb-test-target.exe");
    assert!(p.is_file(), "eupb-test-target.exe missing at {}", p.display());
    p
}

fn run_roundtrip(args: &[&str]) -> Vec<String> {
    let tmp = TempDir::new().expect("tempdir");
    let out = tmp.path().join("result.json");
    let target = target_path();

    let status = Command::cargo_bin("eupb")
        .expect("build eupb")
        .arg("--show-console") // keep things simple for CI; flag is accepted
        .arg("--")
        .arg(&target)
        .arg("--out")
        .arg(&out)
        .args(args)
        .status()
        .expect("spawn eupb");
    assert!(status.success(), "eupb exited with {:?}", status);

    let data = std::fs::read_to_string(&out).expect("read result.json");
    parse_json_string_array(&data)
}

/// Extremely small JSON string-array parser (same format produced by
/// `eupb-test-target`). Avoids a serde_json dependency in the test crate.
fn parse_json_string_array(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();

    // skip whitespace, expect '['
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    assert_eq!(bytes[i] as char, '[', "not a JSON array: {:?}", s);
    i += 1;

    loop {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] as char == ']' {
            break;
        }
        if bytes[i] as char == ',' {
            i += 1;
            continue;
        }
        assert_eq!(bytes[i] as char, '"', "expected string at offset {}", i);
        i += 1;

        let mut cur = String::new();
        while i < bytes.len() && bytes[i] as char != '"' {
            if bytes[i] as char == '\\' {
                i += 1;
                match bytes[i] as char {
                    '"' => cur.push('"'),
                    '\\' => cur.push('\\'),
                    'n' => cur.push('\n'),
                    'r' => cur.push('\r'),
                    't' => cur.push('\t'),
                    'u' => {
                        let hex =
                            std::str::from_utf8(&bytes[i + 1..i + 5]).unwrap();
                        let code = u32::from_str_radix(hex, 16).unwrap();
                        cur.push(char::from_u32(code).unwrap());
                        i += 4;
                    }
                    other => panic!("unsupported escape: \\{}", other),
                }
                i += 1;
            } else {
                // UTF-8: push the whole multi-byte sequence as-is.
                let start = i;
                let first = bytes[i];
                let len = if first < 0x80 {
                    1
                } else if first < 0xC0 {
                    1
                } else if first < 0xE0 {
                    2
                } else if first < 0xF0 {
                    3
                } else {
                    4
                };
                i += len;
                cur.push_str(std::str::from_utf8(&bytes[start..i]).unwrap());
            }
        }
        i += 1; // skip closing quote
        out.push(cur);
    }

    out
}

#[test]
fn target_not_found_exits_with_code_2() {
    let status = Command::cargo_bin("eupb")
        .expect("build eupb")
        .arg("--quiet-errors")
        .arg("--")
        .arg("this-binary-does-not-exist-xyz.exe")
        .status()
        .expect("spawn eupb");
    assert_eq!(status.code(), Some(2));
}

#[test]
fn no_target_exits_with_code_1() {
    let status = Command::cargo_bin("eupb")
        .expect("build eupb")
        .arg("--quiet-errors")
        .status()
        .expect("spawn eupb");
    assert_eq!(status.code(), Some(1));
}

#[test]
fn plain_ascii_roundtrip() {
    let got = run_roundtrip(&["hello", "world"]);
    assert_eq!(got, vec!["hello".to_string(), "world".to_string()]);
}

#[test]
fn args_with_spaces_and_quotes() {
    let got = run_roundtrip(&["hello world", r#"say "hi""#]);
    assert_eq!(
        got,
        vec!["hello world".to_string(), r#"say "hi""#.to_string()]
    );
}

#[test]
fn path_with_trailing_backslash_and_space() {
    let got = run_roundtrip(&[r"C:\My Stuff\"]);
    assert_eq!(got, vec![r"C:\My Stuff\".to_string()]);
}

#[test]
fn unicode_french_accents() {
    let got = run_roundtrip(&["Dossier Été", "café.txt"]);
    assert_eq!(
        got,
        vec!["Dossier Été".to_string(), "café.txt".to_string()]
    );
}

#[test]
fn unicode_cyrillic() {
    let got = run_roundtrip(&[r"C:\Тестовая папка\файл.txt"]);
    assert_eq!(got, vec![r"C:\Тестовая папка\файл.txt".to_string()]);
}

#[test]
fn unicode_cjk() {
    let got = run_roundtrip(&[r"C:\テスト\ファイル.txt"]);
    assert_eq!(got, vec![r"C:\テスト\ファイル.txt".to_string()]);
}

#[test]
fn unicode_emoji() {
    let got = run_roundtrip(&["📁 Dossier", "📄 Fichier.txt"]);
    assert_eq!(
        got,
        vec!["📁 Dossier".to_string(), "📄 Fichier.txt".to_string()]
    );
}

#[test]
fn apostrophe_in_path() {
    let got = run_roundtrip(&[r"C:\L'été de l'année\mon fichier.txt"]);
    assert_eq!(
        got,
        vec![r"C:\L'été de l'année\mon fichier.txt".to_string()]
    );
}

#[test]
fn long_path_prefix() {
    let got = run_roundtrip(&[r"\\?\C:\very\long\path\to\file.txt"]);
    assert_eq!(got, vec![r"\\?\C:\very\long\path\to\file.txt".to_string()]);
}

#[test]
fn many_args() {
    let args = ["--flag", "value1", "-x", "value 2", "positional"];
    let got = run_roundtrip(&args);
    assert_eq!(
        got,
        args.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );
}

#[test]
fn no_wait_returns_immediately_with_exit_0() {
    let tmp = TempDir::new().expect("tempdir");
    let out = tmp.path().join("result.json");
    let target = target_path();

    let status = Command::cargo_bin("eupb")
        .expect("build eupb")
        .arg("--no-wait")
        .arg("--")
        .arg(&target)
        .arg("--out")
        .arg(&out)
        .arg("detached")
        .status()
        .expect("spawn eupb");
    assert_eq!(status.code(), Some(0), "eupb must exit 0 with --no-wait");

    // The target may not have finished yet; give it a moment.
    for _ in 0..40 {
        if out.is_file() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(
        out.is_file(),
        "target did not produce its output after being detached"
    );
}

#[test]
fn log_file_is_written_with_utf8_bom() {
    let tmp = TempDir::new().expect("tempdir");
    let out = tmp.path().join("result.json");
    let log = tmp.path().join("eupb.log");
    let target = target_path();

    let status = Command::cargo_bin("eupb")
        .expect("build eupb")
        .arg("--log")
        .arg(&log)
        .arg("--")
        .arg(&target)
        .arg("--out")
        .arg(&out)
        .arg("café")
        .status()
        .expect("spawn eupb");
    assert!(status.success(), "eupb exited with {:?}", status);

    let bytes = std::fs::read(&log).expect("read log");
    assert_eq!(
        &bytes[..3],
        &[0xEF, 0xBB, 0xBF],
        "log must start with UTF-8 BOM"
    );
    let content = std::str::from_utf8(&bytes[3..]).expect("log body is UTF-8");
    assert!(content.contains("café"), "log must record the arg: {}", content);
}

#[test]
fn cwd_option_is_honored() {
    let tmp = TempDir::new().expect("tempdir");
    let out = tmp.path().join("result.json");
    let target = target_path();

    // Pass an absolute --out so cwd changes don't affect file creation.
    let status = Command::cargo_bin("eupb")
        .expect("build eupb")
        .arg("--cwd")
        .arg(tmp.path())
        .arg("--")
        .arg(&target)
        .arg("--out")
        .arg(&out)
        .arg("dummy")
        .status()
        .expect("spawn eupb");
    assert!(status.success(), "eupb exited with {:?}", status);
    assert!(out.is_file(), "target did not produce its output");
}

#[test]
fn target_resolved_via_path_search() {
    let tmp = TempDir::new().expect("tempdir");
    let out = tmp.path().join("result.json");

    // Copy the test target to a PATH dir under a *bare* name, then launch
    // it without an extension or directory to exercise PATH+PATHEXT search.
    let src = target_path();
    let path_dir = tmp.path();
    let dst = path_dir.join("eupb-path-probe.exe");
    std::fs::copy(&src, &dst).expect("copy probe");

    // Prepend our tempdir to PATH for this child process only.
    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let mut new_path = std::ffi::OsString::from(path_dir.as_os_str());
    new_path.push(";");
    new_path.push(&old_path);

    let status = Command::cargo_bin("eupb")
        .expect("build eupb")
        .env("PATH", new_path)
        .arg("--")
        .arg("eupb-path-probe")
        .arg("--out")
        .arg(&out)
        .arg("ok")
        .status()
        .expect("spawn eupb");
    assert!(status.success(), "eupb exited with {:?}", status);
    let got = parse_json_string_array(&std::fs::read_to_string(&out).unwrap());
    assert_eq!(got, vec!["ok".to_string()]);

    let _ = Path::new(&dst); // silence unused warning on non-windows paths
}
