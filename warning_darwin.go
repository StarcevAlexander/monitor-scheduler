package main

import (
	"strconv"
	"strings"
	"time"
)

func showOffWarning(timeout time.Duration) (time.Duration, error) {
	args := []string{"-e", warningScript, warningText, warningTitle, postponeButton, strconv.Itoa(int(timeout.Seconds())), postponePrompt}
	for _, o := range postponeOptions {
		args = append(args, o.label)
	}
	out, err := toolOutput(osascriptPath, args...)
	if err != nil {
		return 0, err
	}
	choice := strings.TrimSpace(out)
	for _, o := range postponeOptions {
		if o.label == choice {
			return o.delay, nil
		}
	}
	return 0, nil
}
