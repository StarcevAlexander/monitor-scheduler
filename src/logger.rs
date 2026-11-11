use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use chrono::Local;

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

pub fn init(path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    *LOG_FILE.lock().expect("log mutex poisoned") = Some(file);
    Ok(())
}

pub fn write(message: &str) {
    eprintln!("{message}");
    let mut guard = LOG_FILE.lock().expect("log mutex poisoned");
    if let Some(file) = guard.as_mut() {
        let line = format!("{} {}\n", Local::now().format("%Y/%m/%d %H:%M:%S"), message);
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

#[macro_export]
macro_rules! logf {
    ($($arg:tt)*) => {
        $crate::logger::write(&format!($($arg)*))
    };
}
