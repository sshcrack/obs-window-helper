use anyhow::{anyhow, Result as AnyResult};
use windows::Win32::Foundation::HWND;

use crate::{is_blacklisted_window, monitor::get_monitor_id, validators::is_microsoft_internal_exe, window::{get_command_line_args, get_exe, get_product_name, get_title, get_window_class, hwnd_to_monitor, intersects_with_multiple_monitors}};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub obs_id: String,
    pub pid: u32,
    pub title: Option<String>,
    pub class: Option<String>,
    pub product_name: Option<String>,
    pub monitor: Option<String>,
    pub intersects: Option<bool>,
    pub cmd_line: Option<String>,
}

/// Retrieves the OBS window information associated with the given window handle.
///
/// # Arguments
///
/// * `handle` - The handle to the window.
/// * `is_game` - A flag indicating whether a game capture or a window capture is used.
///
/// # Returns
///
/// Returns the OBS window information as struct
///
/// # Errors
///
/// Returns an error if there was a problem retrieving the OBS ID.
pub fn get_window_info(wnd: HWND, is_game: bool) -> AnyResult<WindowInfo> {
    let (proc_id, full_exe) = get_exe(wnd)?;
    let exe = full_exe.file_name().ok_or(anyhow!("Failed to get file name"))?;
    let exe = exe.to_str().ok_or(anyhow!("Failed to convert to str"))?;
    let exe = exe.to_string();

    if is_microsoft_internal_exe(&exe) {
        return Err(anyhow!("Handle is a Microsoft internal exe"));
    }

    if exe == "obs64.exe" {
        return Err(anyhow!("Handle is obs64.exe"));
    }

    if is_game && is_blacklisted_window(&exe) {
        return Err(anyhow!("Handle is blacklisted (game mode)"));
    }

    let title = get_title(wnd).ok();
    let class = get_window_class(wnd).ok();

    let product_name = get_product_name(&full_exe).ok();
    let monitor = hwnd_to_monitor(wnd).ok();
    let intersects = intersects_with_multiple_monitors(wnd).ok();
    let cmd_line = get_command_line_args(wnd).ok();
    let monitor_id = monitor.map(|e| get_monitor_id(e).ok()).flatten();

    Ok(WindowInfo {
        obs_id: "".to_string(),
        pid: proc_id,
        title,
        class,
        product_name,
        monitor: monitor_id,
        intersects,
        cmd_line,
    })
}
