use std::sync::OnceLock;
use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::RECT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::Graphics::Gdi::CreateFontW;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::CLEARTYPE_QUALITY;
use windows::Win32::Graphics::Gdi::CLIP_DEFAULT_PRECIS;
use windows::Win32::Graphics::Gdi::DEFAULT_CHARSET;
use windows::Win32::Graphics::Gdi::FW_NORMAL;
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::Graphics::Gdi::HGDIOBJ;
use windows::Win32::Graphics::Gdi::OUT_DEFAULT_PRECIS;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::HiDpi::SetThreadDpiAwarenessContext;
use windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT;
use windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_SYSTEM_AWARE;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::AdjustWindowRectEx;
use windows::Win32::UI::WindowsAndMessaging::CreateWindowExW;
use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
use windows::Win32::UI::WindowsAndMessaging::DispatchMessageW;
use windows::Win32::UI::WindowsAndMessaging::GetMessageW;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
use windows::Win32::UI::WindowsAndMessaging::IsDialogMessageW;
use windows::Win32::UI::WindowsAndMessaging::LoadCursorW;
use windows::Win32::UI::WindowsAndMessaging::LoadIconW;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
use windows::Win32::UI::WindowsAndMessaging::RegisterClassExW;
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::SetTimer;
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::TranslateMessage;
use windows::Win32::UI::WindowsAndMessaging::HCURSOR;
use windows::Win32::UI::WindowsAndMessaging::HMENU;
use windows::Win32::UI::WindowsAndMessaging::IDC_ARROW;
use windows::Win32::UI::WindowsAndMessaging::IDI_WARNING;
use windows::Win32::UI::WindowsAndMessaging::MSG;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::Win32::UI::WindowsAndMessaging::SYSTEM_METRICS_INDEX;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW;

use crate::constants::POSTPONE_OPTIONS;
use crate::constants_windows::BASE_DPI;
use crate::constants_windows::BN_CLICKED;
use crate::constants_windows::BS_DEFPUSHBUTTON;
use crate::constants_windows::BS_PUSHBUTTON;
use crate::constants_windows::COLOR_BTNFACE;
use crate::constants_windows::ID_CANCEL;
use crate::constants_windows::ID_OK;
use crate::constants_windows::ID_POSTPONE_FIRST;
use crate::constants_windows::POSTPONE_PROMPT;
use crate::constants_windows::SM_CXSCREEN;
use crate::constants_windows::SM_CYSCREEN;
use crate::constants_windows::SS_ICON;
use crate::constants_windows::SS_LEFT;
use crate::constants_windows::STM_SETICON;
use crate::constants_windows::WARNING_BUTTONS_TOP;
use crate::constants_windows::WARNING_BUTTON_GAP;
use crate::constants_windows::WARNING_BUTTON_HEIGHT;
use crate::constants_windows::WARNING_BUTTON_WIDTH;
use crate::constants_windows::WARNING_CLASS_NAME;
use crate::constants_windows::WARNING_CLIENT_HEIGHT;
use crate::constants_windows::WARNING_CLIENT_WIDTH;
use crate::constants_windows::WARNING_FONT_NAME;
use crate::constants_windows::WARNING_FONT_SIZE;
use crate::constants_windows::WARNING_MARGIN;
use crate::constants_windows::WARNING_OK_TEXT;
use crate::constants_windows::WARNING_OK_TOP;
use crate::constants_windows::WARNING_OK_WIDTH;
use crate::constants_windows::WARNING_PROMPT_HEIGHT;
use crate::constants_windows::WARNING_PROMPT_TOP;
use crate::constants_windows::WARNING_TEXT;
use crate::constants_windows::WARNING_TEXT_HEIGHT;
use crate::constants_windows::WARNING_TEXT_LEFT;
use crate::constants_windows::WARNING_TEXT_WIDTH;
use crate::constants_windows::WARNING_TIMER_ID;
use crate::constants_windows::WARNING_TITLE;
use crate::constants_windows::WM_CLOSE;
use crate::constants_windows::WM_COMMAND;
use crate::constants_windows::WM_SETFONT;
use crate::constants_windows::WM_TIMER;
use crate::constants_windows::WS_CAPTION;
use crate::constants_windows::WS_CHILD;
use crate::constants_windows::WS_EX_DLG_MODAL_FRAME;
use crate::constants_windows::WS_EX_TOPMOST;
use crate::constants_windows::WS_GROUP;
use crate::constants_windows::WS_POPUP;
use crate::constants_windows::WS_SYSMENU;
use crate::constants_windows::WS_TABSTOP;
use crate::constants_windows::WS_VISIBLE;
use crate::models::postpone_delay;

static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

struct WarningControl {
    class: &'static str,
    text: &'static str,
    style: u32,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

unsafe extern "system" fn warning_wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_COMMAND => {
            if (wparam.0 >> 16) & 0xFFFF != BN_CLICKED as usize {
                return LRESULT(0);
            }
            PostQuitMessage((wparam.0 & 0xFFFF) as i32);
            LRESULT(0)
        }
        WM_TIMER | WM_CLOSE => {
            PostQuitMessage(ID_CANCEL as i32);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

pub fn show_off_warning(timeout: Duration) -> Result<Duration, String> {
    register_warning_class()?;
    let (dpi, previous_context) = enter_system_dpi();
    let result = run_warning_ui(timeout, dpi);
    if let Some(previous) = previous_context {
        unsafe {
            SetThreadDpiAwarenessContext(previous);
        }
    }
    result.map(postpone_delay_for)
}

fn enter_system_dpi() -> (u32, Option<DPI_AWARENESS_CONTEXT>) {
    let previous = unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE) };
    if previous.0.is_null() {
        return (BASE_DPI, None);
    }
    let dpi = unsafe { GetDpiForSystem() };
    (dpi, Some(previous))
}

fn run_warning_ui(timeout: Duration, dpi: u32) -> Result<usize, String> {
    let scale = |v: i32| v * dpi as i32 / BASE_DPI as i32;
    unsafe {
        let hwnd = create_warning_window(scale)?;
        let font_height = -(WARNING_FONT_SIZE * dpi as i32) / 72;
        let font_name = wide_null(WARNING_FONT_NAME);
        let font: HFONT = CreateFontW(
            font_height,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            PCWSTR(font_name.as_ptr()),
        );
        let mut ok_button = HWND::default();
        for c in warning_controls() {
            let child = create_warning_control(hwnd, &c, scale)?;
            if c.id == ID_OK {
                ok_button = child;
            }
            if c.style == SS_ICON {
                let icon = LoadIconW(None, IDI_WARNING).unwrap_or_default();
                SendMessageW(
                    child,
                    STM_SETICON,
                    Some(WPARAM(icon.0 as usize)),
                    Some(LPARAM(0)),
                );
                continue;
            }
            SendMessageW(
                child,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(0)),
            );
        }
        let timeout_ms = timeout.as_millis().min(u32::MAX as u128) as u32;
        SetTimer(Some(hwnd), WARNING_TIMER_ID, timeout_ms, None);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(ok_button));
        let quit_id = run_warning_loop(hwnd);
        let _ = DestroyWindow(hwnd);
        if !font.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(font.0));
        }
        Ok(quit_id)
    }
}

fn register_warning_class() -> Result<(), String> {
    if CLASS_REGISTERED.get().is_some() {
        return Ok(());
    }
    unsafe {
        let class_name = wide_null(WARNING_CLASS_NAME);
        let cursor: HCURSOR = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(warning_wndproc),
            hCursor: cursor,
            hbrBackground: HBRUSH((COLOR_BTNFACE + 1) as *mut core::ffi::c_void),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hInstance: HINSTANCE::default(),
            ..WNDCLASSEXW::default()
        };
        if RegisterClassExW(&class) == 0 {
            return Err(format!(
                "RegisterClassExW failed with error {}",
                windows::Win32::Foundation::GetLastError().0
            ));
        }
    }
    let _ = CLASS_REGISTERED.set(());
    Ok(())
}

fn create_warning_window(scale: impl Fn(i32) -> i32) -> Result<HWND, String> {
    let class_name = wide_null(WARNING_CLASS_NAME);
    let title = wide_null(WARNING_TITLE);
    let style = WS_POPUP | WS_CAPTION | WS_SYSMENU;
    let ex_style = WS_EX_TOPMOST | WS_EX_DLG_MODAL_FRAME;
    unsafe {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: scale(WARNING_CLIENT_WIDTH),
            bottom: scale(WARNING_CLIENT_HEIGHT),
        };
        AdjustWindowRectEx(
            &mut rect,
            WINDOW_STYLE(style),
            false,
            WINDOW_EX_STYLE(ex_style),
        )
        .map_err(|e| format!("AdjustWindowRectEx: {e}"))?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let screen_width = GetSystemMetrics(SYSTEM_METRICS_INDEX(SM_CXSCREEN));
        let screen_height = GetSystemMetrics(SYSTEM_METRICS_INDEX(SM_CYSCREEN));
        CreateWindowExW(
            WINDOW_EX_STYLE(ex_style),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WINDOW_STYLE(style),
            (screen_width - width) / 2,
            (screen_height - height) / 2,
            width,
            height,
            None,
            None,
            None,
            None,
        )
        .map_err(|e| format!("CreateWindowExW: {e}"))
    }
}

fn create_warning_control(
    parent: HWND,
    c: &WarningControl,
    scale: impl Fn(i32) -> i32,
) -> Result<HWND, String> {
    let class = wide_null(c.class);
    let text = wide_null(c.text);
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class.as_ptr()),
            PCWSTR(text.as_ptr()),
            WINDOW_STYLE(WS_CHILD | WS_VISIBLE | c.style),
            scale(c.x),
            scale(c.y),
            scale(c.width),
            scale(c.height),
            Some(parent),
            Some(HMENU(c.id as *mut core::ffi::c_void)),
            None,
            None,
        )
        .map_err(|e| format!("CreateWindowExW {}: {e}", c.class))
    }
}

fn warning_controls() -> Vec<WarningControl> {
    let mut controls = vec![
        WarningControl {
            class: "STATIC",
            text: "",
            style: SS_ICON,
            id: 0,
            x: WARNING_MARGIN,
            y: WARNING_MARGIN,
            width: 0,
            height: 0,
        },
        WarningControl {
            class: "STATIC",
            text: WARNING_TEXT,
            style: SS_LEFT,
            id: 0,
            x: WARNING_TEXT_LEFT,
            y: WARNING_MARGIN,
            width: WARNING_TEXT_WIDTH,
            height: WARNING_TEXT_HEIGHT,
        },
        WarningControl {
            class: "STATIC",
            text: POSTPONE_PROMPT,
            style: SS_LEFT,
            id: 0,
            x: WARNING_TEXT_LEFT,
            y: WARNING_PROMPT_TOP,
            width: WARNING_TEXT_WIDTH,
            height: WARNING_PROMPT_HEIGHT,
        },
    ];
    for (i, o) in POSTPONE_OPTIONS.iter().enumerate() {
        let mut style = BS_PUSHBUTTON | WS_TABSTOP;
        if i == 0 {
            style |= WS_GROUP;
        }
        controls.push(WarningControl {
            class: "BUTTON",
            text: o.label,
            style,
            id: ID_POSTPONE_FIRST + i,
            x: WARNING_TEXT_LEFT + i as i32 * (WARNING_BUTTON_WIDTH + WARNING_BUTTON_GAP),
            y: WARNING_BUTTONS_TOP,
            width: WARNING_BUTTON_WIDTH,
            height: WARNING_BUTTON_HEIGHT,
        });
    }
    controls.push(WarningControl {
        class: "BUTTON",
        text: WARNING_OK_TEXT,
        style: BS_DEFPUSHBUTTON | WS_TABSTOP,
        id: ID_OK,
        x: WARNING_CLIENT_WIDTH - WARNING_MARGIN - WARNING_OK_WIDTH,
        y: WARNING_OK_TOP,
        width: WARNING_OK_WIDTH,
        height: WARNING_BUTTON_HEIGHT,
    });
    controls
}

unsafe fn run_warning_loop(hwnd: HWND) -> usize {
    let mut msg = MSG::default();
    loop {
        let r = GetMessageW(&mut msg, None, 0, 0);
        if r.0 == 0 {
            return msg.wParam.0;
        }
        if r.0 == -1 {
            return ID_CANCEL;
        }
        if IsDialogMessageW(hwnd, &msg).as_bool() {
            continue;
        }
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

fn postpone_delay_for(id: usize) -> Duration {
    if id < ID_POSTPONE_FIRST || id >= ID_POSTPONE_FIRST + POSTPONE_OPTIONS.len() {
        return Duration::ZERO;
    }
    postpone_delay(id - ID_POSTPONE_FIRST).unwrap_or(Duration::ZERO)
}

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
