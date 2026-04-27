//! Windows-specific launch logic: CreateProcessW, MessageBoxW, error formatting.

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::process::ExitCode;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, HLOCAL, WAIT_OBJECT_0};
use windows::Win32::System::Diagnostics::Debug::{
    FormatMessageW, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
    FORMAT_MESSAGE_IGNORE_INSERTS,
};
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, WaitForSingleObject, CREATE_NO_WINDOW,
    CREATE_UNICODE_ENVIRONMENT, DETACHED_PROCESS, INFINITE, PROCESS_CREATION_FLAGS,
    PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
};

use crate::Resolved;
use eupb::build_command_line;

/// Show a MessageBox. `is_error` toggles the icon.
pub fn message_box(caption: &str, text: &str, is_error: bool) {
    let w_text = to_wide_nul(text);
    let w_caption = to_wide_nul(caption);
    let flags = MB_OK
        | if is_error {
            MB_ICONERROR
        } else {
            MB_ICONINFORMATION
        };
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(w_text.as_ptr()),
            PCWSTR(w_caption.as_ptr()),
            flags,
        );
    }
}

pub fn launch(r: &Resolved) -> ExitCode {
    // Build the command line: "<escaped target> <escaped args...>"
    let target_os: OsString = r.target.as_os_str().to_os_string();
    let cmd_line = build_command_line(&target_os, &r.args);

    // Log (if asked) before spawning so failures still produce a log.
    if let Some(ref log_path) = r.log {
        if let Err(e) = write_log(log_path, r, &cmd_line) {
            if r.show_errors {
                message_box(
                    "eupb — log write failed",
                    &format!("Could not write log file:\n{}", e),
                    true,
                );
            }
            // Non-fatal: continue launching.
        }
    }

    // Convert app name (target) and command line to wide strings.
    let app_name = to_wide_nul(r.target.as_os_str());
    // lpCommandLine must be writable; make a mutable buffer.
    let mut cmd_wide: Vec<u16> = cmd_line.encode_wide().chain(std::iter::once(0)).collect();

    let cwd_wide: Option<Vec<u16>> = r.cwd.as_ref().map(|p| to_wide_nul(p.as_os_str()));

    // Build an overridden environment block if --set-env was used.
    // Must be sorted (Windows requirement for CREATE_UNICODE_ENVIRONMENT).
    let env_block: Option<Vec<u16>> = if r.set_env.is_empty() {
        None
    } else {
        Some(build_env_block(&r.set_env))
    };

    let mut flags: PROCESS_CREATION_FLAGS = CREATE_UNICODE_ENVIRONMENT;
    if r.hide_console {
        // CREATE_NO_WINDOW suppresses the console window for console-subsystem targets.
        // DETACHED_PROCESS (below) takes precedence over it but both are set intentionally:
        // CREATE_NO_WINDOW acts as the fallback when we do wait (no DETACHED_PROCESS),
        // and is harmless when combined with DETACHED_PROCESS.
        flags |= CREATE_NO_WINDOW;
    }
    if !r.wait {
        // DETACHED_PROCESS fully detaches the child from our console session (no window,
        // no inherited console handles). Takes precedence over CREATE_NO_WINDOW.
        flags |= DETACHED_PROCESS;
    }

    let si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();

    let cwd_ptr: PCWSTR = match &cwd_wide {
        Some(v) => PCWSTR(v.as_ptr()),
        None => PCWSTR(std::ptr::null()),
    };

    let env_ptr: Option<*const core::ffi::c_void> = env_block
        .as_ref()
        .map(|b| b.as_ptr() as *const core::ffi::c_void);

    let result = unsafe {
        CreateProcessW(
            PCWSTR(app_name.as_ptr()),
            Some(PWSTR(cmd_wide.as_mut_ptr())),
            None,
            None,
            false,
            flags,
            env_ptr,
            cwd_ptr,
            &si,
            &mut pi,
        )
    };

    if let Err(e) = result {
        if r.show_errors {
            let sys_msg = format_win_error_from_result(&e);
            message_box(
                "eupb — launch failed",
                &format!(
                    "CreateProcessW failed for:\n  {}\n\n{}",
                    r.target.display(),
                    sys_msg
                ),
                true,
            );
        }
        return ExitCode::from(3);
    }

    // Always close the thread handle; we don't need it.
    unsafe {
        let _ = CloseHandle(pi.hThread);
    }

    if !r.wait {
        unsafe {
            let _ = CloseHandle(pi.hProcess);
        }
        return ExitCode::from(0);
    }

    let wait_result = unsafe { WaitForSingleObject(pi.hProcess, INFINITE) };
    if wait_result != WAIT_OBJECT_0 {
        unsafe {
            let _ = CloseHandle(pi.hProcess);
        }
        if r.show_errors {
            message_box(
                "eupb — wait failed",
                "WaitForSingleObject did not signal success.",
                true,
            );
        }
        return ExitCode::from(4);
    }

    let mut exit_code: u32 = 0;
    let got_exit = unsafe { GetExitCodeProcess(pi.hProcess, &mut exit_code as *mut u32) };
    unsafe {
        let _ = CloseHandle(pi.hProcess);
    }

    if got_exit.is_err() {
        if r.show_errors {
            message_box(
                "eupb — exit code failed",
                "GetExitCodeProcess failed.",
                true,
            );
        }
        return ExitCode::from(4);
    }

    // Clamp: ExitCode::from takes u8. Windows exit codes are u32; fold them
    // into a single byte so Explorer/cmd see *something* reasonable, while
    // still mapping 0 → 0 and nonzero → nonzero.
    if exit_code == 0 {
        ExitCode::from(0)
    } else {
        // Preserve the low byte, but guarantee nonzero.
        let byte = (exit_code & 0xFF) as u8;
        ExitCode::from(if byte == 0 { 1 } else { byte })
    }
}

fn to_wide_nul(s: &(impl AsRef<OsStr> + ?Sized)) -> Vec<u16> {
    s.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

/// Build a `CREATE_UNICODE_ENVIRONMENT` block from the parent's current env,
/// merged with the overrides (which win on case-insensitive ASCII name match,
/// matching Windows' own semantics). The block is sorted alphabetically by
/// uppercased name (a hard CreateProcessW requirement) and double-null
/// terminated.
fn build_env_block(overrides: &[(OsString, OsString)]) -> Vec<u16> {
    // (upper_key, original_key, value) — upper_key is the sort & dedup key.
    let mut entries: Vec<(Vec<u16>, Vec<u16>, Vec<u16>)> = Vec::new();

    for (k, v) in std::env::vars_os() {
        let k_wide: Vec<u16> = k.encode_wide().collect();
        if k_wide.is_empty() {
            continue;
        }
        // Skip the Windows legacy "=X:=..." per-drive cwd variables: they
        // start with '=', which CreateProcessW rejects in a sorted block.
        if k_wide[0] == b'=' as u16 {
            continue;
        }
        let v_wide: Vec<u16> = v.encode_wide().collect();
        let upper = ascii_upper_wide(&k_wide);
        entries.push((upper, k_wide, v_wide));
    }

    for (k, v) in overrides {
        let k_wide: Vec<u16> = k.encode_wide().collect();
        let v_wide: Vec<u16> = v.encode_wide().collect();
        let upper = ascii_upper_wide(&k_wide);
        entries.retain(|(u, _, _)| *u != upper);
        entries.push((upper, k_wide, v_wide));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out: Vec<u16> = Vec::new();
    for (_, k, v) in entries {
        out.extend_from_slice(&k);
        out.push(b'=' as u16);
        out.extend_from_slice(&v);
        out.push(0);
    }
    // Double-null terminator (the block's own terminator).
    out.push(0);
    out
}

fn ascii_upper_wide(s: &[u16]) -> Vec<u16> {
    s.iter()
        .map(|&c| {
            if (b'a' as u16..=b'z' as u16).contains(&c) {
                c - (b'a' as u16 - b'A' as u16)
            } else {
                c
            }
        })
        .collect()
}

fn format_win_error_from_result(err: &windows::core::Error) -> String {
    let code = err.code();
    let hresult_val = code.0 as u32;
    // HRESULT from Win32 errors is usually 0x8007NNNN; extract NNNN as WIN32_ERROR.
    let win32_err: u32 = if (hresult_val & 0xFFFF_0000) == 0x8007_0000 {
        hresult_val & 0xFFFF
    } else {
        unsafe { GetLastError().0 }
    };
    let msg =
        format_system_message(win32_err).unwrap_or_else(|| String::from("(no system message)"));
    format!("Error 0x{:08X}: {}", hresult_val, msg.trim_end())
}

fn format_system_message(code: u32) -> Option<String> {
    let mut buf_ptr: PWSTR = PWSTR::null();
    let len = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            None,
            code,
            0,
            // When ALLOCATE_BUFFER is set, lpBuffer is interpreted as PWSTR*.
            PWSTR(&mut buf_ptr as *mut PWSTR as *mut u16),
            0,
            None,
        )
    };
    if len == 0 || buf_ptr.is_null() {
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(buf_ptr.0, len as usize) };
    let s = OsString::from_wide(slice).to_string_lossy().into_owned();
    unsafe {
        let _ = windows::Win32::Foundation::LocalFree(Some(HLOCAL(buf_ptr.0 as _)));
    }
    Some(s)
}

fn write_log(path: &std::path::Path, r: &Resolved, cmd_line: &OsString) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    f.write_all(&[0xEF, 0xBB, 0xBF])?; // UTF-8 BOM

    writeln!(f, "eupb v{}", env!("CARGO_PKG_VERSION"))?;
    writeln!(f, "target      : {}", r.target.display())?;
    writeln!(f, "hide_console: {}", r.hide_console)?;
    writeln!(f, "wait        : {}", r.wait)?;
    if let Some(ref c) = r.cwd {
        writeln!(f, "cwd         : {}", c.display())?;
    }
    writeln!(f, "args ({}):", r.args.len())?;
    for (i, a) in r.args.iter().enumerate() {
        writeln!(f, "  [{}] {}", i, a.to_string_lossy())?;
    }
    if !r.set_env.is_empty() {
        writeln!(f, "set_env ({}):", r.set_env.len())?;
        for (k, v) in &r.set_env {
            writeln!(f, "  {}={}", k.to_string_lossy(), v.to_string_lossy())?;
        }
    }
    writeln!(f, "command line:")?;
    writeln!(f, "  {}", cmd_line.to_string_lossy())?;
    Ok(())
}
