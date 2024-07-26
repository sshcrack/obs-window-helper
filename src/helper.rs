use anyhow::{anyhow, Result as AnyResult};
use windows::Win32::Foundation::HWND;

use crate::{is_blacklisted_window, validators::is_microsoft_internal_exe, window::{get_exe, get_product_name, get_title, get_window_class, hwnd_to_monitor, intersects_with_multiple_monitors}};



/// Retrieves the OBS ID associated with the given window handle.
///
/// # Arguments
///
/// * `handle` - The handle to the window.
/// * `is_game` - A flag indicating whether a game capture or a window capture is used.
///
/// # Returns
///
/// Returns the OBS ID as a string.
///
/// # Errors
///
/// Returns an error if there was a problem retrieving the OBS ID.
pub fn get_obs_id(handle: HWND, is_game: bool) -> AnyResult<String> {
    let (proc_id, full_exe) = get_exe(handle)?;
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

    let str_pid = proc_id.to_string();
    let title = get_title(handle)?;
    let class = get_window_class(handle)?;

    let product_name = get_product_name(&full_exe)?;
    let monitor = hwnd_to_monitor(handle)?;
    let intersects = intersects_with_multiple_monitors(handle)?;

    Ok("OBS ID".to_string())
}
