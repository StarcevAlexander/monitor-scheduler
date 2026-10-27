package main

import (
	"errors"
	"fmt"

	"golang.org/x/sys/windows"
	"golang.org/x/sys/windows/registry"
)

func installAutostart(exePath string) error {
	key, _, err := registry.CreateKey(registry.CURRENT_USER, autostartKeyPath, registry.SET_VALUE)
	if err != nil {
		return fmt.Errorf("open Run key: %w", err)
	}
	defer key.Close()
	if err := key.SetStringValue(autostartValueName, `"`+exePath+`" run`); err != nil {
		return fmt.Errorf("set autostart: %w", err)
	}
	return nil
}

func uninstallAutostart() error {
	key, err := registry.OpenKey(registry.CURRENT_USER, autostartKeyPath, registry.SET_VALUE)
	if err != nil {
		return fmt.Errorf("open Run key: %w", err)
	}
	defer key.Close()
	if err := key.DeleteValue(autostartValueName); err != nil && !errors.Is(err, registry.ErrNotExist) {
		return fmt.Errorf("remove autostart: %w", err)
	}
	return nil
}

func acquireSingleInstance() (func(), error) {
	name, err := windows.UTF16PtrFromString(singleInstanceMutex)
	if err != nil {
		return nil, fmt.Errorf("mutex name: %w", err)
	}
	h, err := windows.CreateMutex(nil, false, name)
	if errors.Is(err, windows.ERROR_ALREADY_EXISTS) {
		windows.CloseHandle(h)
		return nil, errors.New("scheduler is already running")
	}
	if err != nil {
		return nil, fmt.Errorf("create mutex: %w", err)
	}
	return func() { windows.CloseHandle(h) }, nil
}
