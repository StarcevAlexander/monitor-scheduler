package main

const (
	pmsetPath = "/usr/bin/pmset"
	caffeinatePath = "/usr/bin/caffeinate"
	osascriptPath = "/usr/bin/osascript"
	launchAgentLabel = "com.starcevalexander.monitor-scheduler"
	launchAgentsDir = "Library/LaunchAgents"
	singleInstanceLockName = ".monitors.lock"
	desktopDir = "Desktop"
	shortcutExt = ".command"
	shortcutScript = "#!/bin/sh\nexec %s off\n"
	postponeButton = "Отложить…"
)

const warningScript = `on run argv
set answer to display dialog (item 1 of argv) with title (item 2 of argv) buttons {(item 3 of argv), "OK"} default button "OK" with icon caution giving up after ((item 4 of argv) as integer)
if button returned of answer is not (item 3 of argv) then return ""
set picked to choose from list (items 6 thru -1 of argv) with title (item 2 of argv) with prompt (item 5 of argv)
if picked is false then return ""
return item 1 of picked
end run`

const launchAgentPlist =`<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>%s</string>
	<key>ProgramArguments</key>
	<array>
		<string>%s</string>
		<string>run</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
</dict>
</plist>
`
