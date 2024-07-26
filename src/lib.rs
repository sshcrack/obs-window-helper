#[cfg(not(windows))]
compile_error!("This library only supports windows!");

#[cfg(not(target_pointer_width = "64"))]
compile_error!("compilation is only allowed for 64-bit targets");

mod game;
mod util;
mod monitor;
mod window;
mod win_iterator;

pub(crate) use util::*;

#[cfg(test)]
mod test;

pub use game::*;
pub use helper::*;
use validators::WindowSearchMode;
use win_iterator::{first_window, next_window};
use windows::Win32::{Foundation::HWND, System::Console::GetConsoleWindow};

pub fn get_all_windows() -> anyhow::Result<Vec<WindowInfo>> {
    let mode = WindowSearchMode::ExcludeMinimized;
    let check_game = false;
    let mut use_find_window_ex = false;

    let mut parent = None as Option<HWND>;
    let window = unsafe { first_window(mode, &mut parent, &mut use_find_window_ex)? };
    let mut window = Some(window);

    let curr = unsafe { GetConsoleWindow() };

    let mut out = Vec::new();
    while window.is_some_and(|e| !e.is_invalid()) {
        let w = window.unwrap();
        if curr != w {
            let res = get_window_info(w, check_game);
            if let Ok(info) = res {
                out.push(info);
            } else {
                //eprintln!("Error: {:?}", res.err().unwrap());
            }
        }

        unsafe {
            window = next_window(window, mode, &mut parent, use_find_window_ex)?;
        }
    }

    Ok(out)
}

