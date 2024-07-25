use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::GetWindowThreadProcessId};
use windows_result::Result;

pub fn get_exe(handle: HWND, full_path: bool) -> Result<String> {
    let id = None;
    unsafe {
        GetWindowThreadProcessId(wnd, id);
    }

	HANDLE hProc = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE, FALSE, *id);
	if (hProc == 0)
		return false;


	if (GetModuleFileNameEx(hProc, NULL, path_buf, MAX_PATH) == 0)
		return false;

	wstring wexe(path_buf);
	string exe(wexe.begin(), wexe.end());
	CloseHandle(hProc);
	if (fullPath) {
		executable = exe;
		return true;
	}
	fs::path p(exe);

	executable = p.filename().string();

	return true;
}

pub fn get_obs_id(handle: HWND,game_mode: bool) -> Result<String> {
    let full_exe = get_exe(handle, true)?;
}