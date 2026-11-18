use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;

use matui::css;
use matui::Badge;
use matui::Button;
use matui::ButtonVariant;
use matui::Card;
use matui::Render;
use matui::TextField;
use matui::TopAppBar;

use crate::config;
use crate::constants::GUI_WEB_HOST;
use crate::constants::GUI_WEB_PORT;
use crate::constants::POSTPONE_OPTIONS;
use crate::constants::TIMER_FILE_NAME;
use crate::constants::TIMER_TIME_FORMAT;
use crate::constants::WARNING_LEAD;
use crate::logf;
use crate::models::ClockTime;
use crate::monitor;
use crate::scheduler::parse_timer_file;
use crate::scheduler::write_timer_file;

pub fn run_panel(dir: &Path) -> Result<(), String> {
    let listener = match TcpListener::bind((GUI_WEB_HOST, GUI_WEB_PORT)) {
        Ok(l) => l,
        Err(_) => {
            open_in_browser(&panel_url());
            return Ok(());
        }
    };
    logf!("gui: panel listening on http://{GUI_WEB_HOST}:{GUI_WEB_PORT}");
    open_in_browser(&panel_url());
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_conn(stream, dir),
            Err(e) => logf!("gui: connection failed: {e}"),
        }
    }
    Ok(())
}

fn panel_url() -> String {
    format!("http://{GUI_WEB_HOST}:{GUI_WEB_PORT}/")
}

fn open_in_browser(url: &str) {
    let result = open_url(url);
    if let Err(e) = result {
        logf!("gui: open browser failed: {e} ({url})");
    }
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(windows)]
    let command = "cmd";

    let mut cmd = std::process::Command::new(command);
    #[cfg(windows)]
    {
        cmd.args(["/C", "start", "", url]);
    }
    #[cfg(not(windows))]
    {
        cmd.arg(url);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(crate::constants_windows::CREATE_NO_WINDOW);
    }
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

fn handle_conn(stream: TcpStream, dir: &Path) {
    let peer_result = serve(stream, dir);
    if let Err(e) = peer_result {
        logf!("gui: request failed: {e}");
    }
}

fn serve(stream: TcpStream, dir: &Path) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| e.to_string())?;
    let mut header = String::new();
    loop {
        header.clear();
        if reader.read_line(&mut header).map_err(|e| e.to_string())? <= 2 {
            break;
        }
    }
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(format!("malformed request line: {request_line:?}"));
    }
    let target = parts[1].to_string();
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };
    let params = parse_query(&query);
    let mut stream = stream;
    route(&mut stream, dir, &path, &params)
}

fn route(
    stream: &mut TcpStream,
    dir: &Path,
    path: &str,
    params: &[(String, String)],
) -> Result<(), String> {
    match path {
        "/" => {
            let error = param(params, "err");
            let page = render_page(dir, error.as_deref());
            respond_html(stream, &page)
        }
        "/favicon.ico" => respond_empty(stream),
        "/action/off" => {
            monitor::monitors_off()?;
            logf!("gui: monitors off");
            respond_redirect(stream, "/")
        }
        "/action/on" => {
            monitor::monitors_on()?;
            logf!("gui: monitors on");
            respond_redirect(stream, "/")
        }
        "/timer/start" => match start_timer(dir, params) {
            Ok(()) => respond_redirect(stream, "/"),
            Err(e) => {
                logf!("gui: timer start failed: {e}");
                respond_redirect(stream, &error_redirect(&e))
            }
        },
        "/timer/cancel" => {
            let path = timer_path(dir);
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("remove timer file: {e}")),
            }
            logf!("gui: timer cancelled");
            respond_redirect(stream, "/")
        }
        "/schedule" => match save_schedule(dir, params) {
            Ok(()) => respond_redirect(stream, "/"),
            Err(e) => {
                logf!("gui: schedule save failed: {e}");
                respond_redirect(stream, &error_redirect(&e))
            }
        },
        _ => respond_not_found(stream),
    }
}

fn error_redirect(message: &str) -> String {
    format!("/?err={}", url_encode(message))
}

fn url_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn start_timer(dir: &Path, params: &[(String, String)]) -> Result<(), String> {
    let delay_secs: u64 = param(params, "delay")
        .and_then(|v| v.parse().ok())
        .ok_or("unknown delay")?;
    if !POSTPONE_OPTIONS
        .iter()
        .any(|o| o.delay.as_secs() == delay_secs)
    {
        return Err(format!("delay {delay_secs} is not offered"));
    }
    let at = chrono::Local::now() + chrono::Duration::seconds(delay_secs as i64);
    write_timer_file(&timer_path(dir), at)?;
    logf!(
        "gui: timer set, monitors off at {}",
        at.format(TIMER_TIME_FORMAT)
    );
    Ok(())
}

fn save_schedule(dir: &Path, params: &[(String, String)]) -> Result<(), String> {
    let off_text = param(params, "off").unwrap_or_default();
    let on_text = param(params, "on").unwrap_or_default();
    let off = ClockTime::parse(off_text.trim())?;
    let on = ClockTime::parse(on_text.trim())?;
    config::set_rule_times(&dir.join(crate::constants::CONFIG_FILE_NAME), off, on)?;
    logf!("gui: schedule saved");
    Ok(())
}

fn param(params: &[(String, String)], key: &str) -> Option<String> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => (url_decode(k), url_decode(v)),
            None => (url_decode(pair), String::new()),
        })
        .collect()
}

fn url_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn respond_html(stream: &mut TcpStream, page: &str) -> Result<(), String> {
    respond(
        stream,
        "200 OK",
        "text/html; charset=utf-8",
        page.as_bytes(),
    )
}

fn respond_redirect(stream: &mut TcpStream, location: &str) -> Result<(), String> {
    write!(
        stream,
        "HTTP/1.1 303 See Other\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| e.to_string())
}

fn respond_not_found(stream: &mut TcpStream) -> Result<(), String> {
    respond(
        stream,
        "404 Not Found",
        "text/plain; charset=utf-8",
        b"not found",
    )
}

fn respond_empty(stream: &mut TcpStream) -> Result<(), String> {
    write!(
        stream,
        "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| e.to_string())
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

fn render_page(dir: &Path, error: Option<&str>) -> String {
    let (off, on) = config::load(&dir.join(crate::constants::CONFIG_FILE_NAME))
        .map(|cfg| config::rule_times(&cfg))
        .unwrap_or((None, None));
    let timer = std::fs::read_to_string(timer_path(dir))
        .ok()
        .and_then(|raw| parse_timer_file(&raw));
    let scheduler_up = scheduler_running();

    let timer_status = match timer {
        Some(at) => {
            let left = at - chrono::Local::now();
            let minutes = ((left.num_seconds() + 59) / 60).max(0);
            if minutes > 0 {
                format!(
                    "Выключение по таймеру в <span class=\"mono\">{}</span> (осталось ~{} мин)",
                    at.format("%H:%M"),
                    minutes
                )
            } else {
                "Выключение по таймеру…".to_string()
            }
        }
        None => "Таймер не активен".to_string(),
    };
    let scheduler_badge = if scheduler_up {
        Badge::new("планировщик активен").render()
    } else {
        Badge::new("планировщик не запущен").render()
    };

    let mut delay_buttons = String::new();
    for option in POSTPONE_OPTIONS.iter() {
        let href = format!("/timer/start?delay={}", option.delay.as_secs());
        delay_buttons.push_str(
            &Button::new(option.label)
                .variant(ButtonVariant::Outlined)
                .color(matui::Color::Primary)
                .link(href)
                .render(),
        );
    }
    let cancel_button = Button::new("Отменить таймер")
        .variant(ButtonVariant::Text)
        .color(matui::Color::Error)
        .link("/timer/cancel")
        .render();

    let timer_card = Card::new()
        .title("Таймер выключения")
        .body(format!(
            "<p class=\"matui-body-medium mono-status\">{timer_status}</p>\
             <div class=\"monitors-row\">{delay_buttons}</div>\
             <div class=\"monitors-row monitors-row--end\">{cancel_button}</div>"
        ))
        .render();

    let off_button = Button::new("Выключить сейчас")
        .variant(ButtonVariant::Filled)
        .color(matui::Color::Error)
        .link("/action/off")
        .render();
    let on_button = Button::new("Включить сейчас")
        .variant(ButtonVariant::Filled)
        .color(matui::Color::Primary)
        .link("/action/on")
        .render();
    let control_card = Card::new()
        .title("Управление")
        .body(format!(
            "<p class=\"matui-body-medium\">Экраны гаснут сразу; компьютер продолжает работать.</p>\
             <div class=\"monitors-actions\">{off_button}{on_button}</div>"
        ))
        .render();

    let off_field = TextField::new("Выключение в")
        .name("off")
        .value(off.map(|t| t.to_string()).unwrap_or_default())
        .variant(matui::TextFieldVariant::Outlined)
        .helper("чч:мм")
        .render();
    let on_field = TextField::new("Включение в")
        .name("on")
        .value(on.map(|t| t.to_string()).unwrap_or_default())
        .variant(matui::TextFieldVariant::Outlined)
        .helper("чч:мм")
        .render();
    let schedule_card = Card::new()
        .title("Расписание")
        .body(format!(
            "<form method=\"get\" action=\"/schedule\" class=\"monitors-form\">\
             <div class=\"monitors-fields\">{off_field}{on_field}</div>\
             <button class=\"matui-btn matui-btn--filled matui-btn--primary monitors-submit\" type=\"submit\">Сохранить расписание</button>\
             </form>"
        ))
        .render();

    let banner = error
        .map(|e| {
            format!(
                "<div class=\"monitors-error\">Не получилось: {}</div>",
                html_escape(e)
            )
        })
        .unwrap_or_default();

    let schedule_note = format!(
        "<p class=\"matui-body-small monitors-note\">Таймер и расписание подхватывает фоновый планировщик; за {} до выключения появится окно-предупреждение с возможностью отложить.</p>",
        format_duration(WARNING_LEAD)
    );

    format!(
        "<!doctype html><html lang=\"ru\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta http-equiv=\"refresh\" content=\"15\">\
         <title>Мониторы</title><style>{}{}</style></head><body>\
         {}\
         <main class=\"monitors-grid\">{banner}\
         <div class=\"monitors-status\">{scheduler_badge}<span class=\"matui-body-medium mono-status\">{timer_status}</span></div>\
         {control_card}{timer_card}{schedule_card}{schedule_note}</main></body></html>",
        css::stylesheet(),
        overrides_css(),
        TopAppBar::new("Мониторы")
            .actions(format!(
                "<span class=\"monitors-clock mono\">{}</span>",
                chrono::Local::now().format("%H:%M")
            ))
            .render(),
    )
}

fn format_duration(d: std::time::Duration) -> String {
    let minutes = d.as_secs() / 60;
    format!("{minutes} мин")
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn overrides_css() -> &'static str {
    ":root{--matui-primary:#FCEE0A;--matui-on-primary:#101010;--matui-primary-container:#26260F;--matui-on-primary-container:#FCEE0A;--matui-error:#FF2E55;--matui-on-error:#FFFFFF;--matui-error-container:#331016;--matui-on-error-container:#FF8AA5;--matui-secondary-container:#101A1C;--matui-on-secondary-container:#00F0FF;--matui-surface:#0B0C0E;--matui-on-surface:#F5F5F5;--matui-on-surface-variant:#9DA3AE;--matui-outline:#3A3E46;--matui-surface-container:#121418;--matui-surface-container-high:#17191D;}\
     body{background:#0B0C0E;margin:0;padding-bottom:40px;font-family:'Segoe UI',system-ui,sans-serif;}\
     body::before{content:\"\";display:block;height:3px;background:linear-gradient(90deg,#FCEE0A 0 62%,#FF2E55 62% 82%,#00F0FF 82% 100%);}\
     .matui-topbar{background:transparent;padding-top:14px;}\
     .matui-topbar__title{color:var(--matui-on-surface);font-weight:700;}\
     .matui-badge{background:#1A1D14;color:var(--matui-primary);border:1px solid #3A3E46;}\
     .matui-badge--dot{background:var(--matui-primary);}\
     .monitors-grid{display:flex;flex-direction:column;gap:16px;max-width:600px;margin:16px auto;padding:0 16px;}\
     .matui-card{border-radius:6px;border:1px solid var(--matui-outline);}\
     .matui-btn{border-radius:4px;text-transform:uppercase;letter-spacing:.05em;font-weight:700;}\
     .monitors-actions{display:flex;gap:12px;margin-top:12px;}\
     .monitors-actions .matui-btn{flex:1;min-height:58px;font-size:17px;}\
     .monitors-row{display:flex;gap:10px;flex-wrap:wrap;margin-top:12px;}\
     .monitors-row--end{justify-content:flex-end;}\
     .monitors-fields{display:flex;gap:14px;flex-wrap:wrap;margin-top:8px;}\
     .monitors-fields .matui-textfield{flex:1;min-width:150px;}\
     .monitors-submit{margin-top:14px;min-height:46px;}\
     .monitors-status{display:flex;align-items:center;gap:10px;color:var(--matui-on-surface);}\
     .mono,.mono-status,.monitors-clock{font-family:Consolas,'Courier New',monospace;}\
     .monitors-clock{font-size:22px;font-weight:700;color:var(--matui-primary);}\
     .monitors-note{color:var(--matui-on-surface-variant);margin-top:2px;}\
     .monitors-error{border:1px solid var(--matui-error);color:var(--matui-on-error-container);background:var(--matui-error-container);border-radius:6px;padding:10px 14px;font-size:14px;}"
}

fn timer_path(dir: &Path) -> PathBuf {
    dir.join(TIMER_FILE_NAME)
}

fn scheduler_running() -> bool {
    #[cfg(windows)]
    {
        let name = to_wide(crate::constants_windows::SINGLE_INSTANCE_MUTEX);
        unsafe {
            windows::Win32::System::Threading::OpenMutexW(
                windows::Win32::System::Threading::SYNCHRONIZATION_ACCESS_RIGHTS(
                    crate::constants_windows::GUI_MUTEX_ACCESS,
                ),
                false,
                windows::core::PCWSTR(name.as_ptr()),
            )
            .is_ok()
        }
    }
    #[cfg(not(windows))]
    {
        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(_) => return false,
        };
        let lock = exe
            .parent()
            .unwrap_or(Path::new("."))
            .join(crate::constants_darwin::SINGLE_INSTANCE_LOCK_NAME);
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(lock)
        {
            Ok(f) => f,
            Err(_) => return false,
        };
        use std::os::unix::io::AsRawFd;
        let blocked = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) != 0 };
        drop(file);
        blocked
    }
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
