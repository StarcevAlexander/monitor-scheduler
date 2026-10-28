package main

import (
	"fmt"
	"runtime"
	"sync"
	"time"
	"unsafe"

	"golang.org/x/sys/windows"
)

var (
	gdi32 = windows.NewLazySystemDLL("gdi32.dll")
	procRegisterClassExW = user32.NewProc("RegisterClassExW")
	procCreateWindowExW = user32.NewProc("CreateWindowExW")
	procDefWindowProcW = user32.NewProc("DefWindowProcW")
	procDestroyWindow = user32.NewProc("DestroyWindow")
	procShowWindow = user32.NewProc("ShowWindow")
	procSetForegroundWindow = user32.NewProc("SetForegroundWindow")
	procSetFocus = user32.NewProc("SetFocus")
	procGetMessageW = user32.NewProc("GetMessageW")
	procIsDialogMessageW = user32.NewProc("IsDialogMessageW")
	procTranslateMessage = user32.NewProc("TranslateMessage")
	procDispatchMessageW = user32.NewProc("DispatchMessageW")
	procPostQuitMessage = user32.NewProc("PostQuitMessage")
	procSendMessageW = user32.NewProc("SendMessageW")
	procSetTimer = user32.NewProc("SetTimer")
	procLoadIconW = user32.NewProc("LoadIconW")
	procLoadCursorW = user32.NewProc("LoadCursorW")
	procGetSystemMetrics = user32.NewProc("GetSystemMetrics")
	procAdjustWindowRectEx = user32.NewProc("AdjustWindowRectEx")
	procSetThreadDpiAwarenessContext = user32.NewProc("SetThreadDpiAwarenessContext")
	procGetDpiForSystem = user32.NewProc("GetDpiForSystem")
	procCreateFontW = gdi32.NewProc("CreateFontW")
	procDeleteObject = gdi32.NewProc("DeleteObject")
)

var (
	warningWndProc = windows.NewCallback(handleWarningMessage)
	registerWarningClass = sync.OnceValue(func() error {
		className, err := windows.UTF16PtrFromString(warningClassName)
		if err != nil {
			return fmt.Errorf("class name: %w", err)
		}
		cursor, _, _ := procLoadCursorW.Call(0, idcArrow)
		class := wndClassEx{
			wndProc: warningWndProc,
			cursor: cursor,
			background: colorBtnFace + 1,
			className: className,
		}
		class.size = uint32(unsafe.Sizeof(class))
		if r, _, err := procRegisterClassExW.Call(uintptr(unsafe.Pointer(&class))); r == 0 {
			return fmt.Errorf("RegisterClassExW: %w", err)
		}
		return nil
	})
)

func handleWarningMessage(hwnd, message, wParam, lParam uintptr) uintptr {
	switch message {
	case wmCommand:
		if wParam>>16&0xFFFF != bnClicked {
			return 0
		}
		procPostQuitMessage.Call(wParam & 0xFFFF)
		return 0
	case wmTimer, wmClose:
		procPostQuitMessage.Call(idCancel)
		return 0
	}
	r, _, _ := procDefWindowProcW.Call(hwnd, message, wParam, lParam)
	return r
}

func showOffWarning(timeout time.Duration) (time.Duration, error) {
	runtime.LockOSThread()
	defer runtime.UnlockOSThread()
	if err := registerWarningClass(); err != nil {
		return 0, err
	}
	dpi, restoreDPI := enterSystemDPI()
	defer restoreDPI()
	scale := func(v int) uintptr {
		return uintptr(v * int(dpi) / baseDPI)
	}

	hwnd, err := createWarningWindow(scale)
	if err != nil {
		return 0, err
	}
	defer procDestroyWindow.Call(hwnd)

	fontHeight := -warningFontSize * int(dpi) / 72
	fontName, err := windows.UTF16PtrFromString(warningFontName)
	if err != nil {
		return 0, fmt.Errorf("font name: %w", err)
	}
	font, _, _ := procCreateFontW.Call(uintptr(fontHeight), 0, 0, 0, fwNormal, 0, 0, 0, defaultCharset, 0, 0, cleartypeQuality, 0, uintptr(unsafe.Pointer(fontName)))
	if font != 0 {
		defer procDeleteObject.Call(font)
	}
	var okButton uintptr
	for _, c := range warningControls() {
		child, err := createWarningControl(hwnd, c, scale)
		if err != nil {
			return 0, err
		}
		if c.id == idOK {
			okButton = child
		}
		if c.style == ssIcon {
			icon, _, _ := procLoadIconW.Call(0, idiWarning)
			procSendMessageW.Call(child, stmSetIcon, icon, 0)
			continue
		}
		procSendMessageW.Call(child, wmSetFont, font, 0)
	}

	procSetTimer.Call(hwnd, warningTimerID, uintptr(timeout.Milliseconds()), 0)
	procShowWindow.Call(hwnd, swShow)
	procSetForegroundWindow.Call(hwnd)
	procSetFocus.Call(okButton)
	return postponeDelay(runWarningLoop(hwnd)), nil
}

func enterSystemDPI() (uint32, func()) {
	if procSetThreadDpiAwarenessContext.Find() != nil || procGetDpiForSystem.Find() != nil {
		return baseDPI, func() {}
	}
	previous, _, _ := procSetThreadDpiAwarenessContext.Call(dpiAwarenessContextSystemAware)
	dpi, _, _ := procGetDpiForSystem.Call()
	return uint32(dpi), func() {
		procSetThreadDpiAwarenessContext.Call(previous)
	}
}

func createWarningWindow(scale func(int) uintptr) (uintptr, error) {
	className, err := windows.UTF16PtrFromString(warningClassName)
	if err != nil {
		return 0, fmt.Errorf("class name: %w", err)
	}
	title, err := windows.UTF16PtrFromString(warningTitle)
	if err != nil {
		return 0, fmt.Errorf("warning title: %w", err)
	}
	style := wsPopup | wsCaption | wsSysMenu
	exStyle := wsExTopmost | wsExDlgModalFrame
	rect := windows.Rect{Right: int32(scale(warningClientWidth)), Bottom: int32(scale(warningClientHeight))}
	procAdjustWindowRectEx.Call(uintptr(unsafe.Pointer(&rect)), style, 0, exStyle)
	width := int(rect.Right - rect.Left)
	height := int(rect.Bottom - rect.Top)
	screenWidth, _, _ := procGetSystemMetrics.Call(smCxScreen)
	screenHeight, _, _ := procGetSystemMetrics.Call(smCyScreen)
	hwnd, _, callErr := procCreateWindowExW.Call(
		exStyle,
		uintptr(unsafe.Pointer(className)),
		uintptr(unsafe.Pointer(title)),
		style,
		uintptr((int(screenWidth)-width)/2),
		uintptr((int(screenHeight)-height)/2),
		uintptr(width),
		uintptr(height),
		0, 0, 0, 0,
	)
	if hwnd == 0 {
		return 0, fmt.Errorf("CreateWindowExW: %w", callErr)
	}
	return hwnd, nil
}

func warningControls() []warningControl {
	controls := []warningControl{
		{class: "STATIC", style: ssIcon, x: warningMargin, y: warningMargin},
		{class: "STATIC", text: warningText, style: ssLeft, x: warningTextLeft, y: warningMargin, width: warningTextWidth, height: warningTextHeight},
		{class: "STATIC", text: postponePrompt, style: ssLeft, x: warningTextLeft, y: warningPromptTop, width: warningTextWidth, height: warningPromptHeight},
	}
	for i, o := range postponeOptions {
		style := bsPushButton | wsTabStop
		if i == 0 {
			style |= wsGroup
		}
		controls = append(controls, warningControl{
			class: "BUTTON",
			text: o.label,
			style: style,
			id: idPostponeFirst + uintptr(i),
			x: warningTextLeft + i*(warningButtonWidth+warningButtonGap),
			y: warningButtonsTop,
			width: warningButtonWidth,
			height: warningButtonHeight,
		})
	}
	return append(controls, warningControl{
		class: "BUTTON",
		text: warningOKText,
		style: bsDefPushButton | wsTabStop,
		id: idOK,
		x: warningClientWidth - warningMargin - warningOKWidth,
		y: warningOKTop,
		width: warningOKWidth,
		height: warningButtonHeight,
	})
}

func createWarningControl(parent uintptr, c warningControl, scale func(int) uintptr) (uintptr, error) {
	class, err := windows.UTF16PtrFromString(c.class)
	if err != nil {
		return 0, fmt.Errorf("control class: %w", err)
	}
	text, err := windows.UTF16PtrFromString(c.text)
	if err != nil {
		return 0, fmt.Errorf("control text: %w", err)
	}
	hwnd, _, callErr := procCreateWindowExW.Call(
		0,
		uintptr(unsafe.Pointer(class)),
		uintptr(unsafe.Pointer(text)),
		wsChild|wsVisible|c.style,
		scale(c.x),
		scale(c.y),
		scale(c.width),
		scale(c.height),
		parent,
		c.id,
		0, 0,
	)
	if hwnd == 0 {
		return 0, fmt.Errorf("CreateWindowExW %s: %w", c.class, callErr)
	}
	return hwnd, nil
}

func runWarningLoop(hwnd uintptr) uintptr {
	var m msg
	for {
		r, _, _ := procGetMessageW.Call(uintptr(unsafe.Pointer(&m)), 0, 0, 0)
		if r == 0 {
			return m.wParam
		}
		if int32(r) == -1 {
			return idCancel
		}
		if handled, _, _ := procIsDialogMessageW.Call(hwnd, uintptr(unsafe.Pointer(&m))); handled != 0 {
			continue
		}
		procTranslateMessage.Call(uintptr(unsafe.Pointer(&m)))
		procDispatchMessageW.Call(uintptr(unsafe.Pointer(&m)))
	}
}

func postponeDelay(id uintptr) time.Duration {
	if id < idPostponeFirst || id >= idPostponeFirst+uintptr(len(postponeOptions)) {
		return 0
	}
	return postponeOptions[id-idPostponeFirst].delay
}
