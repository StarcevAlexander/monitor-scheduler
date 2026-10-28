package main

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"path/filepath"
	"syscall"

	"golang.org/x/sys/windows"
)

func shortcutPath() (string, error) {
	desktop, err := windows.KnownFolderPath(windows.FOLDERID_Desktop, 0)
	if err != nil {
		return "", fmt.Errorf("locate desktop: %w", err)
	}
	return filepath.Join(desktop, shortcutName+shortcutExt), nil
}

func installShortcut(exePath string) error {
	path, err := shortcutPath()
	if err != nil {
		return err
	}
	system, err := windows.GetSystemDirectory()
	if err != nil {
		return fmt.Errorf("locate system directory: %w", err)
	}
	cmd := exec.Command(filepath.Join(system, powershellRelPath), "-NoProfile", "-NonInteractive", "-Command", shortcutScript)
	cmd.Env = append(os.Environ(),
		shortcutLinkEnv+"="+path,
		shortcutExeEnv+"="+exePath,
		shortcutDirEnv+"="+filepath.Dir(exePath),
		shortcutIconEnv+"="+fmt.Sprintf(shortcutIconFormat, filepath.Join(system, shortcutIconFile), shortcutIconIndex),
	)
	cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true, CreationFlags: windows.CREATE_NO_WINDOW}
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("create shortcut: %w: %s", err, out)
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
