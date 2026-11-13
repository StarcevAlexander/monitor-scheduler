use std::os::windows::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;

use windows::core::PWSTR;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::UI::Shell::FOLDERID_Desktop;
use windows::Win32::UI::Shell::SHGetKnownFolderPath;
use windows::Win32::UI::Shell::KF_FLAG_DEFAULT;

use crate::constants_windows::CREATE_NO_WINDOW;
use crate::constants_windows::POWERSHELL_REL_PATH;
use crate::constants_windows::SHORTCUT_DIR_ENV;
use crate::constants_windows::SHORTCUT_EXE_ENV;
use crate::constants_windows::SHORTCUT_EXT;
use crate::constants_windows::SHORTCUT_ICON_ENV;
use crate::constants_windows::SHORTCUT_ICON_FILE;
use crate::constants_windows::SHORTCUT_ICON_INDEX;
use crate::constants_windows::SHORTCUT_LINK_ENV;
use crate::constants_windows::SHORTCUT_NAME;
use crate::constants_windows::SHORTCUT_SCRIPT;

pub fn install(exe_path: &Path) -> Result<(), String> {
    let link = shortcut_path()?;
    let system = system_dir()?;
    let powershell = system.join(POWERSHELL_REL_PATH);
    let icon = format!(
        "{},{}",
        system.join(SHORTCUT_ICON_FILE).display(),
        SHORTCUT_ICON_INDEX
    );
    let output = std::process::Command::new(&powershell)
        .args(["-NoProfile", "-NonInteractive", "-Command", SHORTCUT_SCRIPT])
        .env(SHORTCUT_LINK_ENV, &link)
        .env(SHORTCUT_EXE_ENV, exe_path)
        .env(
            SHORTCUT_DIR_ENV,
            exe_path.parent().unwrap_or(Path::new(".")),
        )
        .env(SHORTCUT_ICON_ENV, icon)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("create shortcut: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "create shortcut failed: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let path = shortcut_path()?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove shortcut: {e}")),
    }
}

fn shortcut_path() -> Result<PathBuf, String> {
    let desktop = known_desktop()?;
    Ok(desktop.join(format!("{SHORTCUT_NAME}{SHORTCUT_EXT}")))
}

fn known_desktop() -> Result<PathBuf, String> {
    unsafe {
        let path = SHGetKnownFolderPath(&FOLDERID_Desktop, KF_FLAG_DEFAULT, None)
            .map_err(|e| format!("locate desktop: {e}"))?;
        let result = pwstr_to_string(path);
        CoTaskMemFree(Some(path.as_ptr().cast()));
        result
    }
}

unsafe fn pwstr_to_string(path: PWSTR) -> Result<PathBuf, String> {
    let mut len = 0usize;
    while *path.0.add(len) != 0 {
        len += 1;
    }
    let wide = std::slice::from_raw_parts(path.0, len);
    String::from_utf16(wide)
        .map(PathBuf::from)
        .map_err(|e| format!("desktop path: {e}"))
}

fn system_dir() -> Result<PathBuf, String> {
    unsafe {
        let len = GetSystemDirectoryW(None) as usize;
        if len == 0 {
            return Err("locate system directory failed".to_string());
        }
        let mut buffer = vec![0u16; len + 1];
        let written = GetSystemDirectoryW(Some(&mut buffer)) as usize;
        if written == 0 {
            return Err("locate system directory failed".to_string());
        }
        String::from_utf16(&buffer[..written])
            .map(PathBuf::from)
            .map_err(|e| format!("system directory: {e}"))
    }
}
