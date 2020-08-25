mod util;
mod window;

unsafe extern "system" fn enum_windows_callback(
    hwnd: winapi::shared::windef::HWND,
    _: winapi::shared::minwindef::LPARAM,
) -> i32 {
    let wnd = window::Window::from_window_handle(hwnd);
    println!("Window handle {:?}, rect {:?}", hwnd, wnd.get_rect());
    1
}

fn main() {
    pretty_env_logger::init();

    unsafe {
        winapi::um::winuser::EnumWindows(Some(enum_windows_callback), 0);
    }
}
