pub const PMSET_PATH: &str = "/usr/bin/pmset";
pub const CAFFEINATE_PATH: &str = "/usr/bin/caffeinate";
pub const OSASCRIPT_PATH: &str = "/usr/bin/osascript";
pub const LAUNCH_AGENT_LABEL: &str = "com.starcevalexander.monitor-scheduler";
pub const LAUNCH_AGENTS_DIR: &str = "Library/LaunchAgents";
pub const SINGLE_INSTANCE_LOCK_NAME: &str = ".monitors.lock";
pub const DESKTOP_DIR: &str = "Desktop";
pub const SHORTCUT_NAME: &str = "Выключить мониторы";
pub const SHORTCUT_EXT: &str = ".command";
pub const SHORTCUT_SCRIPT: &str = "#!/bin/sh\nexec {} off\n";
pub const POSTPONE_BUTTON: &str = "Отложить…";

pub const WARNING_TITLE: &str = "Мониторы";
pub const WARNING_TEXT: &str = "Мониторы выключатся через 1 минуту.\n\nВключить их снова можно только нажав любую кнопку на клавиатуре.";
pub const POSTPONE_PROMPT: &str = "Отложить выключение на:";

pub const WARNING_SCRIPT: &str = r#"on run argv
set answer to display dialog (item 1 of argv) with title (item 2 of argv) buttons {(item 3 of argv), "OK"} default button "OK" with icon caution giving up after ((item 4 of argv) as integer)
if button returned of answer is not (item 3 of argv) then return ""
set picked to choose from list (items 6 thru -1 of argv) with title (item 2 of argv) with prompt (item 5 of argv)
if picked is false then return ""
return item 1 of picked
end run"#;

pub const LAUNCH_AGENT_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{}</string>
		<string>run</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
</dict>
</plist>
"#;
