use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::Input::KeyboardAndMouse::SendInput;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_KEYBOARD;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_KEYUP;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::constants::PROBE_CAPTURE_SECS;
use crate::constants::PROBE_POLL_MS;
use crate::constants_windows::VK_MEDIA_STOP;
use crate::models::ProbeReading;

const EXTENDED_VKS: [i32; 9] = [0xA6, 0xA7, 0xB0, 0xB1, 0xB2, 0xB3, 0xB5, 0xB6, 0xB7];

fn vk_name(vk: i32) -> &'static str {
    match vk {
        0x0D => "Enter",
        0x13 => "Pause",
        0x14 => "CapsLock",
        0x1B => "Esc",
        0x20 => "Space",
        0x21 => "PageUp",
        0x22 => "PageDown",
        0x23 => "End",
        0x24 => "Home",
        0x2C => "PrintScreen",
        0x2D => "Insert",
        0x70..=0x87 => "F1..F24",
        0x90 => "NumLock",
        0x91 => "ScrollLock",
        0xA6 => "BrowserBack",
        0xA7 => "BrowserForward",
        0xAD => "Mute",
        0xAE => "VolumeDown",
        0xAF => "VolumeUp",
        0xB0 => "MediaNext",
        0xB1 => "MediaPrev",
        0xB2 => "MediaStop",
        0xB3 => "MediaPlayPause",
        0xB4 => "LaunchApp1",
        0xB5 => "LaunchApp2",
        0xB6 => "LaunchMail",
        0xB7 => "LaunchMedia",
        0xC1 => "ReservedC1",
        0xC2 => "ReservedC2",
        0xC3 => "ReservedC3",
        0xC4 => "ReservedC4",
        0xC5 => "ReservedC5",
        0xC6 => "ReservedC6",
        _ => "Unknown",
    }
}

pub fn vk_display_name(vk: i32) -> String {
    let base = vk_name(vk);
    if base == "F1..F24" {
        return format!("F{}", vk - 0x70 + 1);
    }
    if base == "Unknown" {
        return format!("0x{vk:02X}");
    }
    base.to_string()
}

pub fn send_media_stop() -> Result<(), String> {
    send_vk(VK_MEDIA_STOP)
}

fn send_vk(vk: i32) -> Result<(), String> {
    let flags_down = if EXTENDED_VKS.contains(&vk) {
        KEYEVENTF_EXTENDEDKEY
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    let make = |flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk as u16),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let inputs = [make(flags_down), make(KEYEVENTF_KEYUP)];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        return Err(format!("SendInput sent {sent} of {}", inputs.len()));
    }
    Ok(())
}

fn watched_keys() -> HashMap<i32, &'static str> {
    let mut map = HashMap::new();
    for vk in 0x70..=0x87 {
        map.insert(vk, "F1..F24");
    }
    for vk in [
        0x0D, 0x13, 0x14, 0x1B, 0x20, 0x21, 0x22, 0x23, 0x2C, 0x2D, 0x90, 0x91, 0xA6, 0xA7, 0xAD,
        0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xC1, 0xC2, 0xC3,
        0xC4, 0xC5, 0xC6,
    ] {
        map.insert(vk, vk_name(vk));
    }
    map
}

pub fn probe_capture() -> Result<ProbeReading, String> {
    let watch = watched_keys();
    let mut seen: Vec<i32> = Vec::new();
    let mut down: HashMap<i32, bool> = HashMap::new();
    let deadline = Instant::now() + Duration::from_secs(PROBE_CAPTURE_SECS);
    while Instant::now() < deadline {
        for vk in watch.keys() {
            let state = unsafe { GetAsyncKeyState(*vk) };
            let pressed = (state & 0x8000u16 as i16) != 0;
            let was = down.insert(*vk, pressed);
            if pressed && !was.unwrap_or(false) && !seen.contains(vk) {
                seen.push(*vk);
            }
        }
        std::thread::sleep(Duration::from_millis(PROBE_POLL_MS));
    }
    seen.sort_unstable();
    let codes = seen
        .into_iter()
        .map(|vk| format!("{} (VK 0x{vk:02X})", vk_display_name(vk)))
        .collect();
    Ok(ProbeReading { codes })
}
