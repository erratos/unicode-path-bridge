//! Table-driven tests for `escape_arg` (Microsoft C/C++ command-line rules).
//!
//! Each case is the value an application would see via `argv[i]` after
//! `CommandLineToArgvW` re-parses what we produced.

use eupb::escape_arg_str;
use rstest::rstest;

#[rstest]
// Empty and simple cases
#[case::empty("", r#""""#)]
#[case::plain_ascii("foo", "foo")]
#[case::plain_digits("12345", "12345")]
// Spaces and tabs
#[case::space_wraps("hello world", r#""hello world""#)]
#[case::tab_wraps("col\tval", "\"col\tval\"")]
#[case::multiple_spaces("a  b", r#""a  b""#)]
// Backslashes alone (no quotes, no spaces → untouched)
#[case::backslashes_only(r"C:\Users", r"C:\Users")]
#[case::trailing_backslash(r"C:\Users\", r"C:\Users\")]
#[case::double_backslash(r"\\server\share", r"\\server\share")]
// Backslashes with spaces → wrap; backslashes unchanged if not before quote
#[case::backslash_with_space(r"C:\Program Files", r#""C:\Program Files""#)]
#[case::trailing_backslash_with_space(r"C:\My Stuff\", r#""C:\My Stuff\\""#)]
// Quotes: each " becomes \", preceded by doubled backslashes if any
#[case::single_quote_inside(r#"say "hi""#, r#""say \"hi\"""#)]
#[case::leading_quote(r#""quoted""#, r#""\"quoted\"""#)]
#[case::backslash_before_quote(r#"\""#, r#""\\\"""#)]
#[case::two_backslashes_before_quote(r#"\\""#, r#""\\\\\"""#)]
// Mix
#[case::path_with_quote_and_space(r#"C:\foo "bar"\baz"#, r#""C:\foo \"bar\"\baz""#)]
// Unicode passthrough (content doesn't matter for escaping)
#[case::french_accents("café", "café")]
#[case::french_with_space("L'été de l'année", "\"L'été de l'année\"")]
#[case::cyrillic("Тест", "Тест")]
#[case::cyrillic_path_with_space(r"C:\Тест 2024\файл.txt", r#""C:\Тест 2024\файл.txt""#)]
#[case::cjk("日本", "日本")]
#[case::cjk_path(r"C:\テスト\ファイル.txt", r"C:\テスト\ファイル.txt")]
#[case::emoji("🎉", "🎉")]
#[case::emoji_with_space("📁 Dossier", "\"📁 Dossier\"")]
// Apostrophes are not special on the Windows command line
#[case::apostrophe_plain("L'été", "L'été")]
// Long path prefix
#[case::long_path_prefix(r"\\?\C:\very\long\path", r"\\?\C:\very\long\path")]
fn escape_arg_table(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(escape_arg_str(input), expected, "input was {:?}", input);
}

#[test]
fn long_arg_preserved() {
    let long: String = "abcdefghij".repeat(200); // 2000 chars, no specials
    assert_eq!(escape_arg_str(&long), long);
}

#[test]
fn long_arg_with_space_wrapped() {
    let long: String = "word ".repeat(300); // 1500 chars, has spaces
    let escaped = escape_arg_str(&long);
    assert!(escaped.starts_with('"'));
    assert!(escaped.ends_with('"'));
    // The wrapper adds exactly 2 chars.
    assert_eq!(escaped.len(), long.len() + 2);
}

/// Sanity: round-trip through `CommandLineToArgvW` is what escaping is *for*,
/// but that API is Windows-only. On Windows we verify the shape; on other
/// platforms this test compiles but trivially passes.
#[cfg(windows)]
#[test]
fn round_trip_via_commandlinetoargvw() {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::LocalFree;
    use windows::Win32::UI::Shell::CommandLineToArgvW;

    let cases: &[&str] = &[
        r"simple",
        r"hello world",
        r#"say "hi""#,
        r"C:\Program Files\x",
        r"C:\My Stuff\",
        r#"\""#,
        r#"\\""#,
        "café",
        "Тест 2024",
        "🎉 emoji",
    ];

    for orig in cases {
        // Build a synthetic command line: "prog.exe <escaped_arg>"
        let escaped = eupb::escape_arg_str(orig);
        let line = format!("prog.exe {}", escaped);
        let wide: Vec<u16> = OsString::from(&line)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut argc: i32 = 0;
        let argv_ptr = unsafe { CommandLineToArgvW(PCWSTR(wide.as_ptr()), &mut argc) };
        assert!(
            !argv_ptr.is_null(),
            "CommandLineToArgvW failed for {:?}",
            orig
        );
        assert_eq!(argc, 2, "expected 2 args for {:?}", orig);

        let arg1_ptr = unsafe { *argv_ptr.add(1) };
        let mut len = 0usize;
        while unsafe { *arg1_ptr.0.add(len) } != 0 {
            len += 1;
        }
        let slice = unsafe { std::slice::from_raw_parts(arg1_ptr.0, len) };
        let got = OsString::from_wide(slice);
        let _ = unsafe { LocalFree(Some(windows::Win32::Foundation::HLOCAL(argv_ptr as _))) };

        assert_eq!(
            got.to_string_lossy(),
            *orig,
            "round-trip mismatch for {:?} → escaped={:?}",
            orig,
            escaped
        );
    }
}
