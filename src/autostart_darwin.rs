use std::fs;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;

use crate::constants_darwin::LAUNCH_AGENTS_DIR;
use crate::constants_darwin::LAUNCH_AGENT_LABEL;
use crate::constants_darwin::LAUNCH_AGENT_PLIST;
use crate::constants_darwin::SINGLE_INSTANCE_LOCK_NAME;

fn launch_agent_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("locate home failed")?;
    Ok(Path::new(&home)
        .join(LAUNCH_AGENTS_DIR)
        .join(format!("{LAUNCH_AGENT_LABEL}.plist")))
}

pub fn install(exe_path: &Path) -> Result<(), String> {
    let path = launch_agent_path()?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("create LaunchAgents dir: {e}"))?;
    }
    let plist = format!(
        LAUNCH_AGENT_PLIST,
        LAUNCH_AGENT_LABEL,
        escape_xml(&exe_path.display().to_string())
    );
    fs::write(&path, plist).map_err(|e| format!("write launch agent: {e}"))
}

pub fn uninstall() -> Result<(), String> {
    let path = launch_agent_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove launch agent: {e}")),
    }
}

pub struct SingleInstanceGuard {
    _file: fs::File,
}

pub fn acquire_single_instance() -> Result<SingleInstanceGuard, String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("locate executable: {e}"))?;
    let lock_path = exe_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(SINGLE_INSTANCE_LOCK_NAME);
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .read(true)
        .open(&lock_path)
        .map_err(|e| format!("open lock file: {e}"))?;
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        let e = std::io::Error::last_os_error();
        if e.raw_os_error() == Some(libc::EWOULDBLOCK) {
            return Err("scheduler is already running".to_string());
        }
        return Err(format!("lock: {e}"));
    }
    Ok(SingleInstanceGuard { _file: file })
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("&apos;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}
