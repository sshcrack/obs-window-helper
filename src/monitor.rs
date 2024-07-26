use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW};
use windows_result::Result as WinResult;

use crate::string_conv::ToUtf8String;

pub fn get_monitor_id(monitor: HMONITOR) -> WinResult<String> {
    let mut monitor_info = MONITORINFOEXW::default();

    unsafe {
        GetMonitorInfoW(monitor, &mut monitor_info as *mut _ as _).ok()?;
    }

    Ok(monitor_info.szDevice.to_utf8())
}