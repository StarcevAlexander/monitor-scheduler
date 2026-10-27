package main

const (
	pmsetPath = "/usr/bin/pmset"
	caffeinatePath = "/usr/bin/caffeinate"
	launchAgentLabel = "com.starcevalexander.monitor-scheduler"
	launchAgentsDir = "Library/LaunchAgents"
	singleInstanceLockName = ".monitors.lock"
)

const launchAgentPlist = `<?xml version="1.0" encoding="UTF-8"?>
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
