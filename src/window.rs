use std::{
    ffi::OsString,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::string_conv::ToUtf8String;
use anyhow::{anyhow, Result as AnyResult};
use windows::{
    core::HSTRING, Wdk::System::Threading::{NtQueryInformationProcess, ProcessBasicInformation}, Win32::{
        Foundation::{CloseHandle, HANDLE, HMODULE, HWND, MAX_PATH},
        Globalization::GetSystemDefaultLangID,
        Graphics::Gdi::{
            MonitorFromWindow, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL,
        },
        Storage::FileSystem::{
            GetFileVersionInfoExW, GetFileVersionInfoSizeExW, VerQueryValueW, FILE_VER_GET_NEUTRAL,
        },
        System::{
            ProcessStatus::GetModuleFileNameExW,
            Threading::{
                OpenProcess, PROCESS_BASIC_INFORMATION, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ
            },
        },
        UI::WindowsAndMessaging::{
            GetClassNameW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        },
    }
};
use windows_result::{Error, Result as WinResult};

const SZ_STRING_FILE_INFO: &'static str = "StringFileInfo";
const SZ_PRODUCT_NAME: &'static str = "ProductName";
const SZ_HEX_LANG_ID_EN_US: &'static str = "0409";
const SZ_HEX_CODE_PAGE_ID_UNICODE: &'static str = "04B0";

/// Retrieves the executable path and process ID associated with the given window handle.
///
/// # Arguments
///
/// * `handle` - The handle to the window.
///
/// # Returns
///
/// Returns a tuple containing the process ID and the path to the executable.
///
/// # Errors
///
/// Returns an error if there was a problem retrieving the executable path or process ID.
pub fn get_exe(handle: HWND) -> AnyResult<(u32, PathBuf)> {
    let mut proc_id = u32::default();
    unsafe {
        GetWindowThreadProcessId(handle, Some(&mut proc_id));
    }

    let h_proc = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE,
            false,
            proc_id,
        )?
    };

    let exe = unsafe {
        let mut path = [0 as u16; MAX_PATH as usize];
        // HMODULE should be null, not default
        let res = GetModuleFileNameExW(h_proc, HMODULE::default(), &mut path);
        if res > 0 {
            Ok(path.as_ref().to_utf8())
        } else {
            Err(Error::from_win32())
        }
    }?;

    unsafe {
        CloseHandle(h_proc)?;
    }

    Ok((proc_id, PathBuf::from_str(&exe)?))
}

pub fn get_title(handle: HWND) -> AnyResult<String> {
    let len = unsafe { GetWindowTextLengthW(handle) };
    if len == 0 {
        return Err(Error::from_win32().into());
    }

    let len = TryInto::<usize>::try_into(len)?;

    let mut title = vec![0 as u16; len + 1];
    let get_title_res = unsafe { GetWindowTextW(handle, &mut title) };
    if get_title_res == 0 {
        return Err(Error::from_win32().into());
    }

    Ok(title.to_utf8())
}

pub fn get_window_class(handle: HWND) -> WinResult<String> {
    let mut class = [0 as u16; MAX_PATH as usize];

    let len = unsafe { GetClassNameW(handle, &mut class) };
    if len == 0 {
        return Err(Error::from_win32());
    }

    Ok(class.as_ref().to_utf8())
}

pub fn get_product_name(full_exe: &Path) -> AnyResult<String> {
    let exe_wide = HSTRING::from(full_exe.as_os_str());

    let mut dummy = 0;
    let required_buffer_size =
        unsafe { GetFileVersionInfoSizeExW(FILE_VER_GET_NEUTRAL, &exe_wide, &mut dummy) };
    if required_buffer_size == 0 {
        return Err(Error::from_win32().into());
    }

    let mut buffer: Vec<u16> = vec![0; required_buffer_size as usize];
    unsafe {
        GetFileVersionInfoExW(
            FILE_VER_GET_NEUTRAL,
            &exe_wide,
            0,
            required_buffer_size,
            buffer.as_mut_ptr() as *mut _,
        )?;
    }

    let lang_id = unsafe { GetSystemDefaultLangID() };
    let query_key: Vec<u16> = OsString::from(format!(
        "\\{}\\{}{}\\{}",
        SZ_STRING_FILE_INFO, lang_id, SZ_HEX_CODE_PAGE_ID_UNICODE, SZ_PRODUCT_NAME
    ))
    .encode_wide()
    .collect();
    let query_key = HSTRING::from_wide(&query_key)?;

    let mut pages_ptr: *mut u16 = std::ptr::null_mut();
    let mut pages_length = 0;

    unsafe {
        VerQueryValueW(
            buffer.as_mut_ptr() as _,
            &query_key,
            &mut pages_ptr as *mut _ as _,
            &mut pages_length,
        )
        .ok()?
    };

    let chars_in_buf = required_buffer_size / (std::mem::size_of::<u16>() as u32);
    if pages_ptr.is_null() || chars_in_buf < pages_length {
        return Err(anyhow!("Invalid state"));
    }

    let product_name = unsafe { std::slice::from_raw_parts(pages_ptr, pages_length as usize - 1) };
    let product_name = String::from_utf16_lossy(product_name);

    Ok(product_name)
}

pub fn hwnd_to_monitor(handle: HWND) -> WinResult<HMONITOR> {
    unsafe {
        let res = MonitorFromWindow(handle, MONITOR_DEFAULTTONEAREST);
        if res.is_invalid() {
            return Err(Error::from_win32());
        }

        Ok(res)
    }
}

pub fn intersects_with_multiple_monitors(handle: HWND) -> WinResult<bool> {
    unsafe {
        let res = MonitorFromWindow(handle, MONITOR_DEFAULTTONULL);

        return Ok(!res.is_invalid());
    }
}

pub fn get_command_line_args(wnd: HWND) -> WinResult<Vec<String>> {
    let mut proc_id = u32::default();
    unsafe {
        GetWindowThreadProcessId(wnd, Some(&mut proc_id));
    }

    let handle = unsafe {
        OpenProcess(
            PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, //
            false,                                       //
            proc_id,                                     //
        )?
    };

    if handle.is_invalid() {
        return Err(Error::from_win32());
    }

    let res = unsafe { get_command_line_args_priv(handle) };
    unsafe { CloseHandle(handle)?; }

    res
}

unsafe fn get_command_line_args_priv(handle: HANDLE) -> WinResult<Vec<String>> {
    let mut pbi = PROCESS_BASIC_INFORMATION::default();
    // get process information
    NtQueryInformationProcess(
        handle,
        ProcessBasicInformation,
        &mut pbi as *mut _ as _,
        size_of_val(&pbi) as u32,
        std::ptr::null_mut()
    ).ok()?;

    // read PEB
    ReadProcessMemory(handle, pbi.PebBaseAddress, peb, pebSize, NULL);

		// read ProcessParameters
		auto parameters = (PBYTE*)*(LPVOID*)(peb + ProcessParametersOffset); // address in remote process adress space
		if (!ReadProcessMemory(handle, parameters, pp, ppSize, NULL)) {
			CloseHandle(handle);
			return false;
		}

		// read CommandLine
		auto pCommandLine = (UNICODE_STRING*)(pp + CommandLineOffset);
		cmdLine = (PWSTR) new char[pCommandLine->MaximumLength];
		if (!ReadProcessMemory(handle, pCommandLine->Buffer, cmdLine, pCommandLine->MaximumLength, NULL)) {
			CloseHandle(handle);
			return false;
		}
}
