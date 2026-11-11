use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use crate::constants::SHORTCUT_NAME;
use crate::constants_darwin::DESKTOP_DIR;
use crate::constants_darwin::SHORTCUT_EXT;
use crate::constants_darwin::SHORTCUT_SCRIPT;

pub fn install(exe_path: &Path) -> Result<(), String> {
    let path = shortcut_path()?;
    let quoted = format!(
        "'{}'",
        exe_path.display().to_string().replace('\'', "'\\''")
    );
    let script = format!(SHORTCUT_SCRIPT, quoted);
    fs::write(&path, script).map_err(|e| format!("write shortcut: {e}"))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("chmod shortcut: {e}"))
}

pub fn uninstall() -> Result<(), String> {
    let path = shortcut_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove shortcut: {e}")),
    }
}

fn shortcut_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("locate home failed")?;
    Ok(Path::new(&home)
        .join(DESKTOP_DIR)
        .join(format!("{SHORTCUT_NAME}{SHORTCUT_EXT}")))
}
