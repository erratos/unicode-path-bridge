//! eupb — Explorer Unicode Path Bridge
//!
//! Library code for argument escaping, exposed so tests can drive it directly.

#[cfg(windows)]
use std::ffi::{OsStr, OsString};
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};

const SPACE: u16 = b' ' as u16;
const TAB: u16 = b'\t' as u16;
const QUOTE: u16 = b'"' as u16;
const BACKSLASH: u16 = b'\\' as u16;

/// Escape a single argument for the Windows command line.
///
/// Follows the Microsoft C/C++ command-line parsing rules
/// (<https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments>):
///
/// - An argument without spaces, tabs, or double quotes is returned as-is.
/// - Otherwise it is wrapped in `"..."`.
/// - A run of `N` backslashes followed by `"` becomes `2N+1` backslashes plus
///   an escaped quote.
/// - A run of `N` backslashes not followed by `"` stays as `N` backslashes.
/// - A trailing run of backslashes before the closing quote is doubled.
/// - An empty argument becomes `""`.
pub fn escape_arg_wide(arg: &[u16]) -> Vec<u16> {
    if arg.is_empty() {
        return vec![QUOTE, QUOTE];
    }

    let needs_quoting = arg.iter().any(|&c| c == SPACE || c == TAB || c == QUOTE);
    if !needs_quoting {
        return arg.to_vec();
    }

    let mut out: Vec<u16> = Vec::with_capacity(arg.len() + 4);
    out.push(QUOTE);

    let mut backslashes: usize = 0;
    for &c in arg {
        if c == BACKSLASH {
            backslashes += 1;
        } else if c == QUOTE {
            for _ in 0..(backslashes * 2 + 1) {
                out.push(BACKSLASH);
            }
            out.push(QUOTE);
            backslashes = 0;
        } else {
            for _ in 0..backslashes {
                out.push(BACKSLASH);
            }
            out.push(c);
            backslashes = 0;
        }
    }

    // Trailing backslashes before the closing quote must be doubled.
    for _ in 0..(backslashes * 2) {
        out.push(BACKSLASH);
    }
    out.push(QUOTE);
    out
}

#[cfg(windows)]
pub fn escape_arg(arg: &OsStr) -> OsString {
    let wide: Vec<u16> = arg.encode_wide().collect();
    OsString::from_wide(&escape_arg_wide(&wide))
}

/// Build a `CreateProcessW`-style command line from a program path and its args.
/// The program path is escaped just like any other argument.
#[cfg(windows)]
pub fn build_command_line(program: &OsStr, args: &[OsString]) -> OsString {
    let mut out = escape_arg(program);
    for a in args {
        out.push(" ");
        out.push(escape_arg(a));
    }
    out
}

/// Helper used by tests and doc examples — escape a UTF-8 &str and return UTF-8.
pub fn escape_arg_str(arg: &str) -> String {
    let wide: Vec<u16> = arg.encode_utf16().collect();
    String::from_utf16(&escape_arg_wide(&wide))
        .expect("escape_arg_wide preserves validity when input is valid UTF-16 from UTF-8")
}

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn plain_arg_unchanged() {
        assert_eq!(escape_arg_str("foo"), "foo");
    }

    #[test]
    fn empty_arg_becomes_double_quote() {
        assert_eq!(escape_arg_str(""), "\"\"");
    }

    #[test]
    fn space_gets_quoted() {
        assert_eq!(escape_arg_str("hello world"), "\"hello world\"");
    }
}
