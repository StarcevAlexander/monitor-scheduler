package main

import (
	"encoding/xml"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"golang.org/x/sys/unix"
)

func launchAgentPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("locate home: %w", err)
	}
	return filepath.Join(home, launchAgentsDir, launchAgentLabel+".plist"), nil
}

func installAutostart(exePath string) error {
	path, err := launchAgentPath()
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return fmt.Errorf("create LaunchAgents dir: %w", err)
	}
	var escaped strings.Builder
	if err := xml.EscapeText(&escaped, []byte(exePath)); err != nil {
		return fmt.Errorf("escape path: %w", err)
	}
	plist := fmt.Sprintf(launchAgentPlist, launchAgentLabel, escaped.String())
	if err := os.WriteFile(path, []byte(plist), 0o644); err != nil {
		return fmt.Errorf("write launch agent: %w", err)
	}
	return nil
}

func uninstallAutostart() error {
	path, err := launchAgentPath()
	if err != nil {
		return err
	}
	if err := os.Remove(path); err != nil && !errors.Is(err, fs.ErrNotExist) {
		return fmt.Errorf("remove launch agent: %w", err)
	}
	return nil
}

func acquireSingleInstance() (func(), error) {
	exePath, err := os.Executable()
	if err != nil {
		return nil, fmt.Errorf("locate executable: %w", err)
	}
	f, err := os.OpenFile(filepath.Join(filepath.Dir(exePath), singleInstanceLockName), os.O_CREATE|os.O_RDWR, 0o600)
	if err != nil {
		return nil, fmt.Errorf("open lock file: %w", err)
	}
	if err := unix.Flock(int(f.Fd()), unix.LOCK_EX|unix.LOCK_NB); err != nil {
		f.Close()
		if errors.Is(err, unix.EWOULDBLOCK) {
			return nil, errors.New("scheduler is already running")
		}
		return nil, fmt.Errorf("lock: %w", err)
	}
	return func() { f.Close() }, nil
}
