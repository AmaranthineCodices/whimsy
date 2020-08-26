/// Rustified abstraction layer over winapi for interacting with (top-level) windows.
use winapi::um::winuser;

pub type WindowHandle = winapi::shared::windef::HWND;
type Win32Rect = winapi::shared::windef::RECT;

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    fn xyxy(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        let (x0, x1) = if left < right {
            (left, right)
        } else {
            (right, left)
        };

        let (y0, y1) = if top < bottom {
            (top, bottom)
        } else {
            (bottom, top)
        };

        Rect {
            left: x0,
            top: y0,
            right: x1,
            bottom: y1,
        }
    }

    fn xywh(left: i32, top: i32, width: i32, height: i32) -> Rect {
        let right = left + width;
        let bottom = top + height;

        Rect::xyxy(left, top, right, bottom)
    }

    fn wh(&self) -> (i32, i32) {
        (
            (self.right - self.left).abs(),
            (self.bottom - self.top).abs(),
        )
    }
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
            let mut winapi_rect: Win32Rect = std::mem::zeroed();
            evaluate_fallible_winapi!(winuser::GetWindowRect(self.handle, &mut winapi_rect));

            Ok(Rect {
                left: winapi_rect.left,
                top: winapi_rect.top,
                right: winapi_rect.right,
                bottom: winapi_rect.bottom,
            })
        }
    }

    pub fn set_rect(&mut self, rect: Rect) -> Result<(), ()> {
        // Size and position will change, Z order will not. We don't want to activate the window,
        // and this call should be non-blocking.
        let flags = winuser::SWP_NOZORDER | winuser::SWP_NOACTIVATE | winuser::SWP_ASYNCWINDOWPOS;
        let (w, h) = rect.wh();

        unsafe {
            evaluate_fallible_winapi!(winuser::SetWindowPos(
                self.handle,
                std::ptr::null_mut(),
                rect.left,
                rect.top,
                w,
                h,
                flags
            ));
        }

        Ok(())
    }
}
