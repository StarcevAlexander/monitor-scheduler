package main

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
)

func shortcutPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("locate home: %w", err)
	}
	return filepath.Join(home, desktopDir, shortcutName+shortcutExt), nil
}

func installShortcut(exePath string) error {
	path, err := shortcutPath()
	if err != nil {
		return err
	}
	quoted := "'" + strings.ReplaceAll(exePath, "'", `'\''`) + "'"
	if err := os.WriteFile(path, fmt.Appendf(nil, shortcutScript, quoted), 0o755); err != nil {
		return fmt.Errorf("write shortcut: %w", err)
	}
	return nil
}

func uninstallShortcut() error {
	path, err := shortcutPath()
	if err != nil {
		return err
	}
	if err := os.Remove(path); err != nil && !errors.Is(err, fs.ErrNotExist) {
		return fmt.Errorf("remove shortcut: %w", err)
	}
	return nil
}
