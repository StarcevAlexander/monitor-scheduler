package main

const (
	pmsetPath = "/usr/bin/pmset"
	caffeinatePath = "/usr/bin/caffeinate"
	osascriptPath = "/usr/bin/osascript"
	launchAgentLabel = "com.starcevalexander.monitor-scheduler"
	launchAgentsDir = "Library/LaunchAgents"
	singleInstanceLockName = ".monitors.lock"
)

const warningScript = `on run argv
display dialog (item 1 of argv) with title (item 2 of argv) buttons {"OK"} default button "OK" with icon caution giving up after %d
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
