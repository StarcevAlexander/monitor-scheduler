use crate::constants_darwin::CAFFEINATE_PATH;
use crate::constants_darwin::PMSET_PATH;

pub fn monitors_off() -> Result<(), String> {
    run_tool(PMSET_PATH, &["displaysleepnow"])
}

pub fn monitors_on() -> Result<(), String> {
    run_tool(CAFFEINATE_PATH, &["-u", "-t", "1"])
}

pub fn status() -> String {
    "status is not supported on macOS".to_string()
}

pub fn tool_output(path: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(path)
        .args(args)
        .output()
        .map_err(|e| format!("{path}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{path}: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_tool(path: &str, args: &[&str]) -> Result<(), String> {
    tool_output(path, args).map(|_| ())
}
