package main

type point struct {
	x int32
	y int32
}

type msg struct {
	hwnd uintptr
	message uint32
	wParam uintptr
	lParam uintptr
	time uint32
	pt point
	private uint32
}

type wndClassEx struct {
	size uint32
	style uint32
	wndProc uintptr
	clsExtra int32
	wndExtra int32
	instance uintptr
	icon uintptr
	cursor uintptr
	background uintptr
	menuName *uint16
	className *uint16
	iconSm uintptr
}

type warningControl struct {
	class string
	text string
	style uintptr
	id uintptr
	x int
	y int
	width int
	height int
}
