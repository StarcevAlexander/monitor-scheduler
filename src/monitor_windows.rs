use windows::Win32::Devices::Display::DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
use windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO;
use windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO;
use windows::Win32::Devices::Display::DISPLAYCONFIG_TARGET_DEVICE_NAME;
use windows::Win32::Devices::Display::DisplayConfigGetDeviceInfo;
use windows::Win32::Devices::Display::GetDisplayConfigBufferSizes;
use windows::Win32::Devices::Display::QDC_ONLY_ACTIVE_PATHS;
use windows::Win32::Devices::Display::QUERY_DISPLAY_CONFIG_FLAGS;
use windows::Win32::Devices::Display::QueryDisplayConfig;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LUID;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_MOVE;
use windows::Win32::UI::Input::KeyboardAndMouse::mouse_event;
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use crate::constants_windows::HWND_BROADCAST;
use crate::constants_windows::MONITOR_POWER_OFF;
use crate::constants_windows::MONITOR_POWER_ON;
use crate::constants_windows::SC_MONITOR_POWER;
use crate::constants_windows::WM_SYSCOMMAND;

pub fn monitors_off() -> Result<(), String> {
    post_monitor_power(MONITOR_POWER_OFF)
}

pub fn monitors_on() -> Result<(), String> {
    post_monitor_power(MONITOR_POWER_ON)?;
    unsafe {
        mouse_event(MOUSEEVENTF_MOVE, -1, 0, 0, 0);
        mouse_event(MOUSEEVENTF_MOVE, 1, 0, 0, 0);
    }
    Ok(())
}

fn post_monitor_power(state: isize) -> Result<(), String> {
    let result = unsafe {
        PostMessageW(
            Some(HWND(HWND_BROADCAST as *mut core::ffi::c_void)),
            WM_SYSCOMMAND,
            WPARAM(SC_MONITOR_POWER),
            LPARAM(state),
        )
    };
    result.map_err(|e| format!("PostMessageW SC_MONITORPOWER: {e}"))
}

pub fn status() -> String {
    let (paths, modes) = match query_display_config(QDC_ONLY_ACTIVE_PATHS) {
        Ok(v) => v,
        Err(e) => return format!("display query failed: {e}"),
    };
    if paths.is_empty() {
        return "no active displays".to_string();
    }
    let lines: Vec<String> = paths
        .iter()
        .map(|p| {
            let target = path_target(p);
            format!(
                "{} [{}] target {}:{}:{}",
                source_resolution(p, &modes),
                target_name(target),
                target.adapter_lo,
                target.adapter_hi,
                target.target_id
            )
        })
        .collect();
    format!("active displays: {}", lines.join("; "))
}

#[derive(Clone, Copy)]
struct DisplayTarget {
    adapter_lo: u32,
    adapter_hi: u32,
    target_id: u32,
}

fn path_target(p: &DISPLAYCONFIG_PATH_INFO) -> DisplayTarget {
    let adapter = p.targetInfo.adapterId;
    DisplayTarget {
        adapter_lo: adapter.LowPart,
        adapter_hi: adapter.HighPart as u32,
        target_id: p.targetInfo.id,
    }
}

fn source_resolution(path: &DISPLAYCONFIG_PATH_INFO, modes: &[DISPLAYCONFIG_MODE_INFO]) -> String {
    let idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
    match modes.get(idx) {
        Some(m) => {
            let mode = unsafe { m.Anonymous.sourceMode };
            format!("{}x{}", mode.width, mode.height)
        }
        None => "?x?".to_string(),
    }
}

fn target_name(target: DisplayTarget) -> String {
    unsafe {
        let mut name = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
        name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        name.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        name.header.adapterId = LUID {
            LowPart: target.adapter_lo,
            HighPart: target.adapter_hi as i32,
        };
        name.header.id = target.target_id;
        if DisplayConfigGetDeviceInfo(&mut name.header) == 0 {
            let end = name
                .monitorFriendlyDeviceName
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(64);
            return String::from_utf16_lossy(&name.monitorFriendlyDeviceName[..end]);
        }
        String::new()
    }
}

fn query_display_config(
    flags: QUERY_DISPLAY_CONFIG_FLAGS,
) -> Result<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>), String> {
    unsafe {
        let mut num_paths = 0u32;
        let mut num_modes = 0u32;
        win32_check(
            GetDisplayConfigBufferSizes(flags, &mut num_paths, &mut num_modes),
            "GetDisplayConfigBufferSizes",
        )?;
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); num_paths as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); num_modes as usize];
        win32_check(
            QueryDisplayConfig(
                flags,
                &mut num_paths,
                paths.as_mut_ptr(),
                &mut num_modes,
                modes.as_mut_ptr(),
                None,
            ),
            "QueryDisplayConfig",
        )?;
        paths.truncate(num_paths as usize);
        modes.truncate(num_modes as usize);
        Ok((paths, modes))
    }
}

fn win32_check(err: WIN32_ERROR, what: &str) -> Result<(), String> {
    if err == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("{what} failed with error {}", err.0))
    }
}
