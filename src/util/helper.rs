use anyhow::{anyhow, Result as AnyResult};
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::GetWindowThreadProcessId};
use windows_result::{Error, Result as WinResult};

use crate::{
    is_blacklisted_window,
    monitor::get_monitor_id,
    validators::is_microsoft_internal_exe,
    window::{
        get_command_line_args, get_exe, get_product_name, get_title, get_window_class,
        hwnd_to_monitor, intersects_with_multiple_monitors,
    },
};


//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub full_exe: String,
    pub obs_id: String,
    pub handle: HWND,
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
    let exe = full_exe
        .file_name()
        .ok_or(anyhow!("Failed to get file name"))?;
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
    let monitor = Some(hwnd_to_monitor(wnd)?);
    let intersects = intersects_with_multiple_monitors(wnd).ok();
    let cmd_line = get_command_line_args(wnd).ok();
    let monitor_id = monitor.map(|e| get_monitor_id(e).ok()).flatten();

    Ok(WindowInfo {
        full_exe: full_exe.to_string_lossy().to_string(),
        obs_id: "".to_string(),
        handle: wnd,
        pid: proc_id,
        title,
        class,
        product_name,
        monitor: monitor_id,
        intersects,
        cmd_line,
    })
}



pub struct ProcessInfo {
    pub process_id: u32,
    pub thread_id: u32,
}

pub fn get_thread_proc_id(wnd: HWND) -> WinResult<ProcessInfo> {
    let mut proc_id = 0u32;

    let thread_id = unsafe { GetWindowThreadProcessId(wnd, Some(&mut proc_id)) };
    if thread_id == 0 {
        return Err(Error::from_win32());
    }

    Ok(ProcessInfo {
        process_id: proc_id,
        thread_id,
    })
}
