#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod action;
mod config;
mod constants;
mod logger;
mod models;
mod schedule;
mod scheduler;

#[cfg(target_os = "macos")]
mod autostart_darwin;
#[cfg(windows)]
mod autostart_windows;
#[cfg(target_os = "macos")]
mod constants_darwin;
#[cfg(windows)]
mod constants_windows;
#[cfg(windows)]
mod gui_windows;
#[cfg(target_os = "macos")]
mod monitor_darwin;
#[cfg(windows)]
mod monitor_windows;
#[cfg(target_os = "macos")]
mod shortcut_darwin;
#[cfg(windows)]
mod shortcut_windows;
#[cfg(target_os = "macos")]
mod warning_darwin;
#[cfg(windows)]
mod warning_windows;

#[cfg(target_os = "macos")]
pub(crate) use autostart_darwin as autostart;
#[cfg(windows)]
pub(crate) use autostart_windows as autostart;
#[cfg(target_os = "macos")]
pub(crate) use monitor_darwin as monitor;
#[cfg(windows)]
pub(crate) use monitor_windows as monitor;
#[cfg(target_os = "macos")]
pub(crate) use shortcut_darwin as shortcut;
#[cfg(windows)]
pub(crate) use shortcut_windows as shortcut;
#[cfg(target_os = "macos")]
pub(crate) use warning_darwin as warning;
#[cfg(windows)]
pub(crate) use warning_windows as warning;

use std::path::Path;
use std::sync::Arc;

use constants::CONFIG_FILE_NAME;
use constants::LOG_FILE_NAME;
use constants::MANUAL_OFF_DELAY;
use constants::WARNING_LEAD;

fn main() {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("locate executable: {e}");
            std::process::exit(1);
        }
    };
    let dir = exe_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    if let Err(e) = logger::init(&dir.join(LOG_FILE_NAME)) {
        eprintln!("open log: {e}");
        std::process::exit(1);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("run");
    if let Err(e) = run(&exe_path, command) {
        logf!("error: {e}");
        std::process::exit(1);
    }
}

fn run(exe_path: &Path, command: &str) -> Result<(), String> {
    let dir = exe_path.parent().unwrap_or(Path::new("."));
    match command {
        "off" => {
            std::thread::sleep(MANUAL_OFF_DELAY);
            monitor::monitors_off()?;
            logf!("monitors off (manual)");
            Ok(())
        }
        "on" => monitor::monitors_on(),
        "gui" => gui_command(dir),
        "warn" => {
            let delay = warning::show_off_warning(WARNING_LEAD)?;
            logf!("warning closed, postpone: {delay:?}");
            Ok(())
        }
        "run" => run_scheduler(&dir.join(CONFIG_FILE_NAME)),
        "install" => {
            autostart::install(exe_path)?;
            logf!("autostart installed: {}", exe_path.display());
            shortcut::install(exe_path)?;
            logf!("desktop shortcut installed");
            Ok(())
        }
        "uninstall" => {
            autostart::uninstall()?;
            logf!("autostart removed");
            shortcut::uninstall()?;
            logf!("desktop shortcut removed");
            Ok(())
        }
        "status" => {
            logf!("{}", monitor::status());
            Ok(())
        }
        "--version" | "version" => {
            logf!("monitors {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => Err(format!(
            "unknown command {command:?}, use: run | gui | on | off | warn | install | uninstall | status"
        )),
    }
}

#[cfg(windows)]
fn gui_command(dir: &Path) -> Result<(), String> {
    gui_windows::run_panel(dir)
}

#[cfg(not(windows))]
fn gui_command(_dir: &Path) -> Result<(), String> {
    Err("gui is supported on Windows only".to_string())
}

fn run_scheduler(config_path: &Path) -> Result<(), String> {
    let _guard = autostart::acquire_single_instance()?;
    let mut s = scheduler::Scheduler::new(
        config_path,
        Box::new(action::apply_action),
        Arc::new(warning::show_off_warning),
    )?;
    logf!("scheduler started, config: {}", config_path.display());
    s.run();
    Ok(())
}
