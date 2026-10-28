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

const (
	wmClose uintptr = 0x0010
	wmSetFont uintptr = 0x0030
	wmCommand uintptr = 0x0111
	wmTimer uintptr = 0x0113
	stmSetIcon uintptr = 0x0170
	wsPopup uintptr = 0x80000000
	wsVisible uintptr = 0x10000000
	wsCaption uintptr = 0x00C00000
	wsSysMenu uintptr = 0x00080000
	wsChild uintptr = 0x40000000
	wsTabStop uintptr = 0x00010000
	wsGroup uintptr = 0x00020000
	wsExTopmost uintptr = 0x00000008
	wsExDlgModalFrame uintptr = 0x00000001
	bsPushButton uintptr = 0x0
	bsDefPushButton uintptr = 0x1
	bnClicked uintptr = 0
	ssLeft uintptr = 0x0
	ssIcon uintptr = 0x3
	swShow uintptr = 5
	smCxScreen uintptr = 0
	smCyScreen uintptr = 1
	colorBtnFace uintptr = 15
	idOK uintptr = 1
	idCancel uintptr = 2
	idPostponeFirst uintptr = 100
	idiWarning uintptr = 32515
	idcArrow uintptr = 32512
	fwNormal uintptr = 400
	defaultCharset uintptr = 1
	cleartypeQuality uintptr = 5
	dpiAwarenessContextSystemAware = ^uintptr(1)
	baseDPI = 96
	warningTimerID uintptr = 1
	warningClassName = "MonitorSchedulerWarning"
	warningFontName = "Segoe UI"
	warningFontSize = 9
	warningOKText = "OK"
)

const (
	warningMargin = 16
	warningClientWidth = 460
	warningTextLeft = 64
	warningTextWidth = 380
	warningTextHeight = 80
	warningPromptTop = 104
	warningPromptHeight = 20
	warningButtonsTop = 130
	warningButtonWidth = 86
	warningButtonHeight = 28
	warningButtonGap = 8
	warningOKTop = 176
	warningOKWidth = 90
	warningClientHeight = warningOKTop + warningButtonHeight + warningMargin
)

const (
	shortcutExt = ".lnk"
	shortcutIconFile = "imageres.dll"
	shortcutIconFormat = "%s,%d"
	shortcutIconIndex = 193
	shortcutLinkEnv = "MONITORS_LINK"
	shortcutExeEnv = "MONITORS_EXE"
	shortcutDirEnv = "MONITORS_DIR"
	shortcutIconEnv = "MONITORS_ICON"
	shortcutScript = `$s = (New-Object -ComObject WScript.Shell).CreateShortcut($env:MONITORS_LINK); $s.TargetPath = $env:MONITORS_EXE; $s.Arguments = 'off'; $s.WorkingDirectory = $env:MONITORS_DIR; $s.IconLocation = $env:MONITORS_ICON; $s.Save()`
	powershellRelPath = `WindowsPowerShell\v1.0\powershell.exe`
)
