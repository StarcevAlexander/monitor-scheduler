use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

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
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::Graphics::Gdi::HGDIOBJ;
use windows::Win32::Graphics::Gdi::OUT_DEFAULT_PRECIS;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::AdjustWindowRectEx;
use windows::Win32::UI::WindowsAndMessaging::CreateWindowExW;
use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
use windows::Win32::UI::WindowsAndMessaging::DispatchMessageW;
use windows::Win32::UI::WindowsAndMessaging::GetMessageW;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::IsDialogMessageW;
use windows::Win32::UI::WindowsAndMessaging::LoadCursorW;
use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
use windows::Win32::UI::WindowsAndMessaging::RegisterClassExW;
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::SetTimer;
use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::Win32::UI::WindowsAndMessaging::TranslateMessage;
use windows::Win32::UI::WindowsAndMessaging::HCURSOR;
use windows::Win32::UI::WindowsAndMessaging::HMENU;
use windows::Win32::UI::WindowsAndMessaging::IDC_ARROW;
use windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR;
use windows::Win32::UI::WindowsAndMessaging::MB_OK;
use windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE;
use windows::Win32::UI::WindowsAndMessaging::MSG;
use windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN;
use windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW;

use crate::config;
use crate::constants::CONFIG_FILE_NAME;
use crate::constants::POSTPONE_OPTIONS;
use crate::constants::TIMER_FILE_NAME;
use crate::constants::TIMER_TIME_FORMAT;
use crate::constants_windows::BASE_DPI;
use crate::constants_windows::BS_DEFPUSHBUTTON;
use crate::constants_windows::BS_PUSHBUTTON;
use crate::constants_windows::COLOR_BTNFACE;
use crate::constants_windows::GUI_ACTION_H;
use crate::constants_windows::GUI_ACTION_W;
use crate::constants_windows::GUI_ACTION_Y;
use crate::constants_windows::GUI_BUTTON_GAP;
use crate::constants_windows::GUI_CLASS_NAME;
use crate::constants_windows::GUI_CLIENT_HEIGHT;
use crate::constants_windows::GUI_CLIENT_WIDTH;
use crate::constants_windows::GUI_EDIT_H;
use crate::constants_windows::GUI_EDIT_ROW_Y;
use crate::constants_windows::GUI_ES_AUTOHSCROLL;
use crate::constants_windows::GUI_FONT_ACTION_PT;
use crate::constants_windows::GUI_FONT_BUTTON_PT;
use crate::constants_windows::GUI_FONT_EDIT_PT;
use crate::constants_windows::GUI_FONT_TEXT_PT;
use crate::constants_windows::GUI_FULL_WIDTH;
use crate::constants_windows::GUI_NOT_RUNNING_SUFFIX;
use crate::constants_windows::GUI_OFF_EDIT_ID;
use crate::constants_windows::GUI_OFF_EDIT_W;
use crate::constants_windows::GUI_OFF_EDIT_X;
use crate::constants_windows::GUI_OFF_LABEL;
use crate::constants_windows::GUI_OFF_LABEL_ID;
use crate::constants_windows::GUI_OFF_LABEL_W;
use crate::constants_windows::GUI_OFF_LABEL_X;
use crate::constants_windows::GUI_OFF_NOW_BUTTON;
use crate::constants_windows::GUI_OFF_NOW_ID;
use crate::constants_windows::GUI_ON_EDIT_ID;
use crate::constants_windows::GUI_ON_EDIT_W;
use crate::constants_windows::GUI_ON_EDIT_X;
use crate::constants_windows::GUI_ON_LABEL;
use crate::constants_windows::GUI_ON_LABEL_ID;
use crate::constants_windows::GUI_ON_LABEL_W;
use crate::constants_windows::GUI_ON_LABEL_X;
use crate::constants_windows::GUI_ON_NOW_BUTTON;
use crate::constants_windows::GUI_ON_NOW_ID;
use crate::constants_windows::GUI_REFRESH_INTERVAL_MS;
use crate::constants_windows::GUI_REFRESH_TIMER_ID;
use crate::constants_windows::GUI_SAVE_BUTTON;
use crate::constants_windows::GUI_SAVE_H;
use crate::constants_windows::GUI_SAVE_ID;
use crate::constants_windows::GUI_SAVE_W;
use crate::constants_windows::GUI_SAVE_Y;
use crate::constants_windows::GUI_SCHEDULE_PROMPT;
use crate::constants_windows::GUI_SCHEDULE_PROMPT_ID;
use crate::constants_windows::GUI_SCHEDULE_PROMPT_Y;
use crate::constants_windows::GUI_SS_RIGHT;
use crate::constants_windows::GUI_SUMMARY_ID;
use crate::constants_windows::GUI_SUMMARY_Y;
use crate::constants_windows::GUI_TIMER_BUTTONS_Y;
use crate::constants_windows::GUI_TIMER_BUTTON_H;
use crate::constants_windows::GUI_TIMER_BUTTON_W;
use crate::constants_windows::GUI_TIMER_CANCEL;
use crate::constants_windows::GUI_TIMER_CANCEL_H;
use crate::constants_windows::GUI_TIMER_CANCEL_ID;
use crate::constants_windows::GUI_TIMER_CANCEL_W;
use crate::constants_windows::GUI_TIMER_CANCEL_Y;
use crate::constants_windows::GUI_TIMER_FIRST_ID;
use crate::constants_windows::GUI_TIMER_IDLE;
use crate::constants_windows::GUI_TIMER_PROMPT;
use crate::constants_windows::GUI_TIMER_PROMPT_ID;
use crate::constants_windows::GUI_TIMER_PROMPT_Y;
use crate::constants_windows::GUI_TIMER_STATUS_ID;
use crate::constants_windows::GUI_TIMER_STATUS_Y;
use crate::constants_windows::GUI_WINDOW_TITLE;
use crate::constants_windows::SINGLE_INSTANCE_MUTEX;
use crate::constants_windows::SS_LEFT;
use crate::constants_windows::WARNING_MARGIN;
use crate::constants_windows::WM_CLOSE;
use crate::constants_windows::WM_COMMAND;
use crate::constants_windows::WM_DESTROY;
use crate::constants_windows::WM_SETFONT;
use crate::constants_windows::WM_TIMER;
use crate::constants_windows::WS_CHILD;
use crate::constants_windows::WS_OVERLAPPEDWINDOW;
use crate::constants_windows::WS_TABSTOP;
use crate::constants_windows::WS_VISIBLE;
use crate::logf;
use crate::models::ClockTime;
use crate::monitor;
use crate::scheduler::parse_timer_file;
use crate::scheduler::write_timer_file;

struct Fonts {
    action: HFONT,
    button: HFONT,
    text: HFONT,
    edit: HFONT,
}

impl Drop for Fonts {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.action.0));
            let _ = DeleteObject(HGDIOBJ(self.button.0));
            let _ = DeleteObject(HGDIOBJ(self.text.0));
            let _ = DeleteObject(HGDIOBJ(self.edit.0));
        }
    }
}

struct GuiContext {
    dir: PathBuf,
    controls: HashMap<usize, HWND>,
}

unsafe impl Send for GuiContext {}

static CONTEXT: OnceLock<Mutex<GuiContext>> = OnceLock::new();

struct ControlSpec {
    class: &'static str,
    text: &'static str,
    style: u32,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

unsafe extern "system" fn panel_wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_COMMAND => {
            let id = wparam.0 & 0xFFFF;
            handle_command(id);
            LRESULT(0)
        }
        WM_TIMER => {
            refresh_timer_status();
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

pub fn run_panel(dir: &Path) -> Result<(), String> {
    register_panel_class()?;
    let hwnd = create_panel_window()?;
    let fonts = Fonts {
        action: make_font(GUI_FONT_ACTION_PT),
        button: make_font(GUI_FONT_BUTTON_PT),
        text: make_font(GUI_FONT_TEXT_PT),
        edit: make_font(GUI_FONT_EDIT_PT),
    };
    let mut controls = HashMap::new();
    for spec in panel_controls() {
        let child = create_control(hwnd, &spec)?;
        apply_font(child, &spec, &fonts);
        controls.insert(spec.id, child);
    }
    let _ = CONTEXT.set(Mutex::new(GuiContext {
        dir: dir.to_path_buf(),
        controls,
    }));
    fill_from_config(dir);
    refresh_timer_status();
    mark_scheduler_state();
    unsafe {
        SetTimer(
            Some(hwnd),
            GUI_REFRESH_TIMER_ID,
            GUI_REFRESH_INTERVAL_MS,
            None,
        );
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(None);
        run_message_loop(hwnd);
        let _ = DestroyWindow(hwnd);
    }
    Ok(())
}

fn register_panel_class() -> Result<(), String> {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    if REGISTERED.get().is_some() {
        return Ok(());
    }
    unsafe {
        let class_name = wide(GUI_CLASS_NAME);
        let cursor: HCURSOR = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(panel_wndproc),
            hCursor: cursor,
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(
                (COLOR_BTNFACE + 1) as *mut core::ffi::c_void,
            ),
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
    let _ = REGISTERED.set(());
    Ok(())
}

fn create_panel_window() -> Result<HWND, String> {
    let class_name = wide(GUI_CLASS_NAME);
    let title = wide(GUI_WINDOW_TITLE);
    let style = WS_OVERLAPPEDWINDOW;
    unsafe {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: GUI_CLIENT_WIDTH,
            bottom: GUI_CLIENT_HEIGHT,
        };
        AdjustWindowRectEx(&mut rect, WINDOW_STYLE(style), false, WINDOW_EX_STYLE(0))
            .map_err(|e| format!("AdjustWindowRectEx: {e}"))?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WINDOW_STYLE(style),
            (screen_w - width) / 2,
            (screen_h - height) / 2,
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

fn panel_controls() -> Vec<ControlSpec> {
    let mut specs = vec![
        ControlSpec {
            class: "STATIC",
            text: "",
            style: SS_LEFT,
            id: GUI_SUMMARY_ID,
            x: WARNING_MARGIN,
            y: GUI_SUMMARY_Y,
            width: GUI_FULL_WIDTH,
            height: 28,
        },
        ControlSpec {
            class: "BUTTON",
            text: GUI_OFF_NOW_BUTTON,
            style: BS_DEFPUSHBUTTON | WS_TABSTOP,
            id: GUI_OFF_NOW_ID,
            x: WARNING_MARGIN,
            y: GUI_ACTION_Y,
            width: GUI_ACTION_W,
            height: GUI_ACTION_H,
        },
        ControlSpec {
            class: "BUTTON",
            text: GUI_ON_NOW_BUTTON,
            style: BS_PUSHBUTTON | WS_TABSTOP,
            id: GUI_ON_NOW_ID,
            x: GUI_CLIENT_WIDTH - WARNING_MARGIN - GUI_ACTION_W,
            y: GUI_ACTION_Y,
            width: GUI_ACTION_W,
            height: GUI_ACTION_H,
        },
        ControlSpec {
            class: "STATIC",
            text: GUI_TIMER_PROMPT,
            style: SS_LEFT,
            id: GUI_TIMER_PROMPT_ID,
            x: WARNING_MARGIN,
            y: GUI_TIMER_PROMPT_Y,
            width: GUI_FULL_WIDTH,
            height: 28,
        },
    ];
    for (i, option) in POSTPONE_OPTIONS.iter().enumerate() {
        specs.push(ControlSpec {
            class: "BUTTON",
            text: option.label,
            style: BS_PUSHBUTTON | WS_TABSTOP,
            id: GUI_TIMER_FIRST_ID + i,
            x: WARNING_MARGIN + i as i32 * (GUI_TIMER_BUTTON_W + GUI_BUTTON_GAP),
            y: GUI_TIMER_BUTTONS_Y,
            width: GUI_TIMER_BUTTON_W,
            height: GUI_TIMER_BUTTON_H,
        });
    }
    specs.extend([
        ControlSpec {
            class: "STATIC",
            text: GUI_TIMER_IDLE,
            style: SS_LEFT,
            id: GUI_TIMER_STATUS_ID,
            x: WARNING_MARGIN,
            y: GUI_TIMER_STATUS_Y,
            width: GUI_FULL_WIDTH,
            height: 28,
        },
        ControlSpec {
            class: "BUTTON",
            text: GUI_TIMER_CANCEL,
            style: BS_PUSHBUTTON | WS_TABSTOP,
            id: GUI_TIMER_CANCEL_ID,
            x: WARNING_MARGIN,
            y: GUI_TIMER_CANCEL_Y,
            width: GUI_TIMER_CANCEL_W,
            height: GUI_TIMER_CANCEL_H,
        },
        ControlSpec {
            class: "STATIC",
            text: GUI_SCHEDULE_PROMPT,
            style: SS_LEFT,
            id: GUI_SCHEDULE_PROMPT_ID,
            x: WARNING_MARGIN,
            y: GUI_SCHEDULE_PROMPT_Y,
            width: GUI_FULL_WIDTH,
            height: 28,
        },
        ControlSpec {
            class: "STATIC",
            text: GUI_OFF_LABEL,
            style: GUI_SS_RIGHT,
            id: GUI_OFF_LABEL_ID,
            x: GUI_OFF_LABEL_X,
            y: GUI_EDIT_ROW_Y,
            width: GUI_OFF_LABEL_W,
            height: GUI_EDIT_H,
        },
        ControlSpec {
            class: "EDIT",
            text: "",
            style: GUI_ES_AUTOHSCROLL | WS_TABSTOP,
            id: GUI_OFF_EDIT_ID,
            x: GUI_OFF_EDIT_X,
            y: GUI_EDIT_ROW_Y,
            width: GUI_OFF_EDIT_W,
            height: GUI_EDIT_H,
        },
        ControlSpec {
            class: "STATIC",
            text: GUI_ON_LABEL,
            style: GUI_SS_RIGHT,
            id: GUI_ON_LABEL_ID,
            x: GUI_ON_LABEL_X,
            y: GUI_EDIT_ROW_Y,
            width: GUI_ON_LABEL_W,
            height: GUI_EDIT_H,
        },
        ControlSpec {
            class: "EDIT",
            text: "",
            style: GUI_ES_AUTOHSCROLL | WS_TABSTOP,
            id: GUI_ON_EDIT_ID,
            x: GUI_ON_EDIT_X,
            y: GUI_EDIT_ROW_Y,
            width: GUI_ON_EDIT_W,
            height: GUI_EDIT_H,
        },
        ControlSpec {
            class: "BUTTON",
            text: GUI_SAVE_BUTTON,
            style: BS_PUSHBUTTON | WS_TABSTOP,
            id: GUI_SAVE_ID,
            x: WARNING_MARGIN,
            y: GUI_SAVE_Y,
            width: GUI_SAVE_W,
            height: GUI_SAVE_H,
        },
    ]);
    specs
}

fn create_control(parent: HWND, spec: &ControlSpec) -> Result<HWND, String> {
    let class = wide(spec.class);
    let text = wide(spec.text);
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class.as_ptr()),
            PCWSTR(text.as_ptr()),
            WINDOW_STYLE(WS_CHILD | WS_VISIBLE | spec.style),
            spec.x,
            spec.y,
            spec.width,
            spec.height,
            Some(parent),
            Some(HMENU(spec.id as *mut core::ffi::c_void)),
            None,
            None,
        )
        .map_err(|e| format!("CreateWindowExW {}: {e}", spec.class))
    }
}

fn apply_font(hwnd: HWND, spec: &ControlSpec, fonts: &Fonts) {
    let font = match spec.id {
        GUI_OFF_NOW_ID | GUI_ON_NOW_ID => fonts.action,
        GUI_TIMER_FIRST_ID..=GUI_TIMER_CANCEL_ID if spec.class == "BUTTON" => fonts.button,
        GUI_OFF_EDIT_ID | GUI_ON_EDIT_ID => fonts.edit,
        GUI_SUMMARY_ID | GUI_SAVE_ID => fonts.text,
        _ => fonts.text,
    };
    unsafe {
        SendMessageW(
            hwnd,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
}

fn make_font(point_size: i32) -> HFONT {
    let name = wide("Segoe UI");
    let height = -point_size * BASE_DPI as i32 / 72;
    unsafe {
        CreateFontW(
            height,
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
            PCWSTR(name.as_ptr()),
        )
    }
}

unsafe fn run_message_loop(hwnd: HWND) {
    let mut msg = MSG::default();
    loop {
        let r = GetMessageW(&mut msg, None, 0, 0);
        if r.0 == 0 {
            return;
        }
        if r.0 == -1 {
            return;
        }
        if IsDialogMessageW(hwnd, &msg).as_bool() {
            continue;
        }
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

fn with_context<T>(f: impl FnOnce(&mut GuiContext) -> T) -> Option<T> {
    let cell = CONTEXT.get()?;
    let mut guard = cell.lock().expect("gui context mutex poisoned");
    Some(f(&mut guard))
}

fn handle_command(id: usize) {
    match id {
        GUI_OFF_NOW_ID => {
            if let Err(e) = monitor::monitors_off() {
                show_error(&e);
                return;
            }
            logf!("gui: monitors off");
        }
        GUI_ON_NOW_ID => {
            if let Err(e) = monitor::monitors_on() {
                show_error(&e);
                return;
            }
            logf!("gui: monitors on");
        }
        GUI_TIMER_FIRST_ID..=GUI_TIMER_CANCEL_ID
            if id < GUI_TIMER_FIRST_ID + POSTPONE_OPTIONS.len() =>
        {
            start_timer(id - GUI_TIMER_FIRST_ID);
        }
        GUI_TIMER_CANCEL_ID => cancel_timer(),
        GUI_SAVE_ID => save_schedule(),
        _ => {}
    }
}

fn start_timer(option_index: usize) {
    let Some(delay) = POSTPONE_OPTIONS.get(option_index).map(|o| o.delay) else {
        return;
    };
    let at =
        chrono::Local::now() + chrono::Duration::from_std(delay).expect("timer delay out of range");
    let outcome = with_context(|ctx| {
        let path = timer_path(&ctx.dir);
        write_timer_file(&path, at)
    });
    match outcome {
        Some(Ok(())) => logf!(
            "gui: timer set, monitors off at {}",
            at.format(TIMER_TIME_FORMAT)
        ),
        Some(Err(e)) => show_error(&e),
        None => {}
    }
    refresh_timer_status();
}

fn cancel_timer() {
    let removed = with_context(|ctx| {
        let path = timer_path(&ctx.dir);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("remove timer file: {e}")),
        }
    });
    match removed {
        Some(Ok(())) => logf!("gui: timer cancelled"),
        Some(Err(e)) => show_error(&e),
        None => {}
    }
    refresh_timer_status();
}

fn save_schedule() {
    let outcome = with_context(|ctx| -> Result<(), String> {
        let off_text = control_text(&ctx.controls, GUI_OFF_EDIT_ID);
        let on_text = control_text(&ctx.controls, GUI_ON_EDIT_ID);
        let off = ClockTime::parse(off_text.trim())?;
        let on = ClockTime::parse(on_text.trim())?;
        let path = ctx.dir.join(CONFIG_FILE_NAME);
        config::set_rule_times(&path, off, on)?;
        Ok(())
    });
    match outcome {
        Some(Ok(())) => {
            logf!("gui: schedule saved");
            update_summary();
        }
        Some(Err(e)) => show_error(&format!("Некорректное время или файл.\n\n{e}")),
        None => {}
    }
}

fn fill_from_config(dir: &Path) {
    let path = dir.join(CONFIG_FILE_NAME);
    let (off, on) = match config::load(&path) {
        Ok(cfg) => config::rule_times(&cfg),
        Err(e) => {
            show_error(&e);
            return;
        }
    };
    with_context(|ctx| {
        if let Some(t) = off {
            set_control_text(&ctx.controls, GUI_OFF_EDIT_ID, &t.to_string());
        }
        if let Some(t) = on {
            set_control_text(&ctx.controls, GUI_ON_EDIT_ID, &t.to_string());
        }
    });
    update_summary();
}

fn update_summary() {
    let summary = with_context(|ctx| {
        let path = ctx.dir.join(CONFIG_FILE_NAME);
        let parsed = config::load(&path).ok();
        let (off, on) = parsed
            .as_ref()
            .map(config::rule_times)
            .unwrap_or((None, None));
        let text = match (off, on) {
            (Some(off), Some(on)) => {
                format!("Расписание: выключение в {off}, включение в {on}")
            }
            (Some(off), None) => format!("Расписание: выключение в {off}"),
            (None, Some(on)) => format!("Расписание: включение в {on}"),
            (None, None) => "Расписание не задано".to_string(),
        };
        let hwnd = ctx.controls.get(&GUI_SUMMARY_ID).copied();
        (text, hwnd)
    });
    if let Some((text, Some(hwnd))) = summary {
        set_text(hwnd, &text);
    }
}

fn mark_scheduler_state() {
    let running = scheduler_running();
    if running {
        return;
    }
    with_context(|ctx| {
        if let Some(hwnd) = ctx.controls.get(&GUI_SUMMARY_ID).copied() {
            let mut text = get_text(hwnd);
            text.push_str(GUI_NOT_RUNNING_SUFFIX);
            set_text(hwnd, &text);
        }
    });
}

fn scheduler_running() -> bool {
    let name = wide(SINGLE_INSTANCE_MUTEX);
    unsafe {
        windows::Win32::System::Threading::OpenMutexW(
            windows::Win32::System::Threading::SYNCHRONIZATION_ACCESS_RIGHTS(
                crate::constants_windows::GUI_MUTEX_ACCESS,
            ),
            false,
            PCWSTR(name.as_ptr()),
        )
        .is_ok()
    }
}

fn refresh_timer_status() {
    let status = with_context(|ctx| {
        let path = timer_path(&ctx.dir);
        let text = match std::fs::read_to_string(&path) {
            Ok(raw) => match parse_timer_file(&raw) {
                Some(at) => {
                    let left = at - chrono::Local::now();
                    let minutes = ((left.num_seconds() + 59) / 60).max(0);
                    if minutes > 0 {
                        format!(
                            "Выключение по таймеру в {} (осталось ~{} мин)",
                            at.format("%H:%M"),
                            minutes
                        )
                    } else {
                        "Выключение по таймеру…".to_string()
                    }
                }
                None => GUI_TIMER_IDLE.to_string(),
            },
            Err(_) => GUI_TIMER_IDLE.to_string(),
        };
        let hwnd = ctx.controls.get(&GUI_TIMER_STATUS_ID).copied();
        (text, hwnd)
    });
    if let Some((text, Some(hwnd))) = status {
        set_text(hwnd, &text);
    }
}

fn timer_path(dir: &Path) -> PathBuf {
    dir.join(TIMER_FILE_NAME)
}

fn control_text(controls: &HashMap<usize, HWND>, id: usize) -> String {
    controls
        .get(&id)
        .map(|hwnd| get_text(*hwnd))
        .unwrap_or_default()
}

fn set_control_text(controls: &HashMap<usize, HWND>, id: usize, text: &str) {
    if let Some(hwnd) = controls.get(&id) {
        set_text(*hwnd, text);
    }
}

fn get_text(hwnd: HWND) -> String {
    unsafe {
        let mut buf = [0u16; 64];
        let len = GetWindowTextW(hwnd, &mut buf);
        let len = (len as usize).min(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }
}

fn set_text(hwnd: HWND, text: &str) {
    let wide = wide(text);
    unsafe {
        let _ = SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
    }
}

fn show_error(message: &str) {
    let text = wide(message);
    let title = wide(GUI_WINDOW_TITLE);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MESSAGEBOX_STYLE((MB_OK | MB_ICONERROR).0),
        );
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
