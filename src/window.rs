/// Rustified abstraction layer over winapi for interacting with (top-level) windows.
use winapi::um::winuser::GetWindowRect;

pub type WindowHandle = winapi::shared::windef::HWND;

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Debug)]
pub struct Window {
    handle: WindowHandle,
}

impl Window {
    pub fn from_window_handle(handle: WindowHandle) -> Window {
        Window { handle }
    }

    pub fn get_rect(&self) -> Result<Rect, ()> {
        unsafe {
            let mut winapi_rect: winapi::shared::windef::RECT = std::mem::zeroed();
            evaluate_fallible_winapi!(GetWindowRect(self.handle, &mut winapi_rect));

            Ok(Rect {
                left: winapi_rect.left,
                top: winapi_rect.top,
                right: winapi_rect.right,
                bottom: winapi_rect.bottom,
            })
        }
    }
}
