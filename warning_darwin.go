package main

import "fmt"

func showOffWarning() error {
	script := fmt.Sprintf(warningScript, int(warningLead.Seconds()))
	return runTool(osascriptPath, "-e", script, warningText, warningTitle)
}
