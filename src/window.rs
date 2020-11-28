/// Rustified abstraction layer over winapi for interacting with (top-level) windows.
use winapi::um::winuser;

use crate::config::Direction;

type WindowHandle = winapi::shared::windef::HWND;
type MonitorHandle = winapi::shared::windef::HMONITOR;
type Win32Rect = winapi::shared::windef::RECT;

/// This type will be returned from methods that invoke fallible Win32 errors.
/// The error, if any, is logged to the output if logging is enabled, but it is
/// not exposed because it is not particularly useful to callers.
pub type Win32Result<T> = Result<T, ()>;

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn xyxy(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
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

    pub fn xywh(left: i32, top: i32, width: i32, height: i32) -> Rect {
        let right = left + width;
        let bottom = top + height;

        Rect::xyxy(left, top, right, bottom)
    }

    pub fn wh(&self) -> (i32, i32) {
        (
            (self.right - self.left).abs(),
            (self.bottom - self.top).abs(),
        )
    }

    pub fn slice_rect(&self, direction: Direction, slice_factor: f32) -> Rect {
        let (width, height) = self.wh();
        let width_slice = ((width as f32) / slice_factor) as i32;
        let height_slice = ((height as f32) / slice_factor) as i32;

        match direction {
            Direction::Up => Rect::xywh(self.left, self.top, width, height_slice),
            Direction::Left => Rect::xywh(self.left, self.top, width_slice, height),
            Direction::Right => Rect::xywh(
                self.left + width - width_slice,
                self.top,
                width_slice,
                height,
            ),
            Direction::Down => Rect::xywh(
                self.left,
                self.top + height - height_slice,
                width,
                height_slice,
            ),
        }
    }

    fn from_win32_rect(rect: Win32Rect) -> Rect {
        Rect::xyxy(rect.left, rect.top, rect.right, rect.bottom)
    }
}

#[derive(Debug)]
pub struct Window {
    handle: WindowHandle,
}

impl Window {
    fn from_window_handle(handle: WindowHandle) -> Window {
        Window { handle }
    }

    pub fn get_rect(&self) -> Win32Result<Rect> {
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

    pub fn set_rect(&mut self, rect: Rect) -> Win32Result<()> {
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

    pub fn get_monitor(&self) -> Monitor {
        unsafe {
            Monitor {
                handle: winuser::MonitorFromWindow(self.handle, winuser::MONITOR_DEFAULTTONEAREST),
            }
        }
    }
}

pub fn get_focused_window() -> Option<Window> {
    unsafe {
        let handle = winuser::GetForegroundWindow();

        if !handle.is_null() {
            Some(Window::from_window_handle(handle))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Monitor {
    handle: MonitorHandle,
}

impl Monitor {
    pub fn get_work_area(&self) -> Win32Result<Rect> {
        unsafe {
            let mut monitor_info: winuser::MONITORINFO = std::mem::zeroed();
            // u32 cast is safe, usize will be 32 bits or larger on all platforms we care about.
            monitor_info.cbSize = std::mem::size_of::<winuser::MONITORINFO>() as u32;

            evaluate_fallible_winapi!(winuser::GetMonitorInfoW(self.handle, &mut monitor_info));
            Ok(Rect::from_win32_rect(monitor_info.rcWork))
        }
    }
}
