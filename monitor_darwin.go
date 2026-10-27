package main

import (
	"bytes"
	"fmt"
	"os/exec"
)

func monitorsOff() error {
	return runTool(pmsetPath, "displaysleepnow")
}

func monitorsOn() error {
	return runTool(caffeinatePath, "-u", "-t", "1")
}

func runTool(path string, args ...string) error {
	out, err := exec.Command(path, args...).CombinedOutput()
	if err != nil {
		return fmt.Errorf("%s: %w: %s", path, err, bytes.TrimSpace(out))
	}
	return nil
}
