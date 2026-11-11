use std::time::Duration;

use crate::constants::POSTPONE_OPTIONS;
use crate::constants_darwin::OSASCRIPT_PATH;
use crate::constants_darwin::POSTPONE_BUTTON;
use crate::constants_darwin::POSTPONE_PROMPT;
use crate::constants_darwin::WARNING_SCRIPT;
use crate::constants_darwin::WARNING_TEXT;
use crate::constants_darwin::WARNING_TITLE;
use crate::monitor::tool_output;

pub fn show_off_warning(timeout: Duration) -> Result<Duration, String> {
    let timeout_seconds = timeout.as_secs().to_string();
    let mut args: Vec<String> = vec![
        "-e".into(),
        WARNING_SCRIPT.into(),
        WARNING_TEXT.into(),
        WARNING_TITLE.into(),
        POSTPONE_BUTTON.into(),
        timeout_seconds,
        POSTPONE_PROMPT.into(),
    ];
    for o in POSTPONE_OPTIONS.iter() {
        args.push(o.label.into());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = tool_output(OSASCRIPT_PATH, &refs)?;
    let choice = out.trim();
    POSTPONE_OPTIONS
        .iter()
        .find(|o| o.label == choice)
        .map(|o| o.delay)
        .unwrap_or(Duration::ZERO)
}
