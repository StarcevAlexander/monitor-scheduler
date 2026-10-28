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
	_, err := toolOutput(path, args...)
	return err
}

func toolOutput(path string, args ...string) (string, error) {
	var stdout, stderr bytes.Buffer
	cmd := exec.Command(path, args...)
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		return "", fmt.Errorf("%s: %w: %s", path, err, bytes.TrimSpace(stderr.Bytes()))
	}
	return stdout.String(), nil
}
