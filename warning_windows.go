package main

import (
	"fmt"
	"unsafe"

	"golang.org/x/sys/windows"
)

var procMessageBoxTimeoutW = user32.NewProc("MessageBoxTimeoutW")

func showOffWarning() error {
	if err := procMessageBoxTimeoutW.Find(); err != nil {
		return fmt.Errorf("MessageBoxTimeoutW: %w", err)
	}
	text, err := windows.UTF16PtrFromString(warningText)
	if err != nil {
		return fmt.Errorf("warning text: %w", err)
	}
	title, err := windows.UTF16PtrFromString(warningTitle)
	if err != nil {
		return fmt.Errorf("warning title: %w", err)
	}
	r, _, callErr := procMessageBoxTimeoutW.Call(
		0,
		uintptr(unsafe.Pointer(text)),
		uintptr(unsafe.Pointer(title)),
		mbIconWarning|mbSystemModal|mbSetForeground|mbTopmost,
		0,
		uintptr(warningLead.Milliseconds()),
	)
	if r == 0 {
		return fmt.Errorf("MessageBoxTimeoutW: %w", callErr)
	}
	return nil
}
