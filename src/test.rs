use crate::{get_all_windows, get_window_info, get_window_info_test, monitor::get_monitor_id, validators::WindowSearchMode, win_iterator::first_window, WindowInfo};

/*
#[test]
pub fn test_iteration() {
    let windows = get_all_windows().unwrap();
    #[cfg(feature="serde")]
    {
        let json = serde_json::to_string_pretty(&windows).unwrap();
        println!("{}", json)
    }
    #[cfg(not(feature="serde"))]
    println!("{:?}", windows);
}
 */

#[test]
pub fn test_single() {
    let pid = 3472;

    let windows = get_all_windows().unwrap()
    .into_iter()
    .find(|e| e.pid == pid)
    .unwrap();

    #[cfg(feature="serde")]
    {
        let info = get_window_info_test(windows.handle, false).unwrap();
        println!("{:#?}", info)
    }
    #[cfg(not(feature="serde"))]
    println!("{:?}", windows);
    }