package main

import (
	"fmt"

	"golang.org/x/sys/windows"
)

var (
	user32 = windows.NewLazySystemDLL("user32.dll")
	procPostMessageW = user32.NewProc("PostMessageW")
	procMouseEvent = user32.NewProc("mouse_event")
)

func monitorsOff() error {
	return postMonitorPower(monitorPowerOff)
}

func monitorsOn() error {
	if err := postMonitorPower(monitorPowerOn); err != nil {
		return err
	}
	up := int32(-1)
	procMouseEvent.Call(mouseEventMove, 0, uintptr(uint32(up)), 0, 0)
	procMouseEvent.Call(mouseEventMove, 0, 1, 0, 0)
	return nil
}

func postMonitorPower(state uintptr) error {
	r, _, err := procPostMessageW.Call(hwndBroadcast, wmSysCommand, scMonitorPower, state)
	if r == 0 {
		return fmt.Errorf("PostMessageW SC_MONITORPOWER: %w", err)
	}
	return nil
}
