use std::path::Path;

use windows::core::HSTRING;
use windows::core::PCWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Registry::RegDeleteKeyValueW;
use windows::Win32::System::Registry::RegSetKeyValueW;
use windows::Win32::System::Registry::HKEY_CURRENT_USER;
use windows::Win32::System::Registry::REG_SZ;
use windows::Win32::System::Threading::CreateMutexW;

use crate::constants_windows::AUTOSTART_KEY_PATH;
use crate::constants_windows::AUTOSTART_VALUE_NAME;
use crate::constants_windows::SINGLE_INSTANCE_MUTEX;

pub fn install(exe_path: &Path) -> Result<(), String> {
    let subkey = HSTRING::from(AUTOSTART_KEY_PATH);
    let value_name = HSTRING::from(AUTOSTART_VALUE_NAME);
    let data = HSTRING::from(format!("\"{}\" run", exe_path.display()));
    let err = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value_name.as_ptr()),
            REG_SZ.0,
            Some(data.as_ptr().cast()),
            ((data.len() + 1) * 2) as u32,
        )
    };
    if err.0 != 0 {
        return Err(format!("set autostart failed with error {}", err.0));
    }
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let subkey = HSTRING::from(AUTOSTART_KEY_PATH);
    let value_name = HSTRING::from(AUTOSTART_VALUE_NAME);
    let err = unsafe {
        RegDeleteKeyValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value_name.as_ptr()),
        )
    };
    if err.0 != 0 && err != ERROR_FILE_NOT_FOUND {
        return Err(format!("remove autostart failed with error {}", err.0));
    }
    Ok(())
}

pub struct SingleInstanceGuard {
    handle: HANDLE,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

pub fn acquire_single_instance() -> Result<SingleInstanceGuard, String> {
    let name = HSTRING::from(SINGLE_INSTANCE_MUTEX);
    let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }
        .map_err(|e| format!("create mutex: {e}"))?;
    let last = unsafe { GetLastError() };
    if last == ERROR_ALREADY_EXISTS {
        let _ = unsafe { CloseHandle(handle) };
        return Err("scheduler is already running".to_string());
    }
    Ok(SingleInstanceGuard { handle })
}
