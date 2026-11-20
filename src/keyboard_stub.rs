use crate::models::ProbeReading;

pub fn send_media_stop() -> Result<(), String> {
    Err("keyboard control is supported on Windows only".to_string())
}

pub fn probe_capture() -> Result<ProbeReading, String> {
    Err("keyboard probe is supported on Windows only".to_string())
}

pub fn vk_display_name(vk: i32) -> String {
    format!("0x{vk:02X}")
}
