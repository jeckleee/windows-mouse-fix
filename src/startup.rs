use std::{
    os::windows::ffi::OsStrExt,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{Foundation::*, System::Registry::*};

const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const NAME: &str = "WindowsMouseFix";

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn check(code: u32) -> Result<(), String> {
    if code == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(code as i32).to_string())
    }
}

pub fn enabled() -> Result<bool, String> {
    let mut size = 0;
    let code = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            wide(RUN).as_ptr(),
            wide(NAME).as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            null_mut(),
            &mut size,
        )
    };
    if code == ERROR_FILE_NOT_FOUND {
        return Ok(false);
    }
    check(code)?;
    Ok(size > 2)
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    // Quote the executable path so spaces and non-ASCII names work without a shell.
    let command = if enabled {
        let path = std::env::current_exe().map_err(|e| e.to_string())?;
        let command: Vec<u16> = std::iter::once(34)
            .chain(path.as_os_str().encode_wide())
            .chain("\" --startup".encode_utf16())
            .chain(Some(0))
            .collect();
        if command.len() > 260 {
            return Err("程序路径过长，请移到较短的路径后再开启自启。".into());
        }
        command
    } else {
        Vec::new()
    };
    unsafe {
        let mut key = null_mut();
        let code = if enabled {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                wide(RUN).as_ptr(),
                0,
                null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                null(),
                &mut key,
                null_mut(),
            )
        } else {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                wide(RUN).as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut key,
            )
        };
        if !enabled && code == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        check(code)?;
        let code = if enabled {
            RegSetValueExW(
                key,
                wide(NAME).as_ptr(),
                0,
                REG_SZ,
                command.as_ptr().cast(),
                (command.len() * 2) as u32,
            )
        } else {
            RegDeleteValueW(key, wide(NAME).as_ptr())
        };
        RegCloseKey(key);
        if !enabled && code == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        check(code)
    }
}
