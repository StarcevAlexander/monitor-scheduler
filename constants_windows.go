package main

const (
	autostartKeyPath = `Software\Microsoft\Windows\CurrentVersion\Run`
	autostartValueName = "MonitorScheduler"
	singleInstanceMutex = `Local\MonitorSchedulerSingleInstance`
)

const (
	hwndBroadcast uintptr = 0xFFFF
	wmSysCommand uintptr = 0x0112
	scMonitorPower uintptr = 0xF170
	monitorPowerOff uintptr = 2
	monitorPowerOn = ^uintptr(0)
	mouseEventMove uintptr = 0x0001
)
