// util before all others due to the macros it contains
mod util;

mod keybind;
mod window;

#[derive(Debug, Copy, Clone)]
enum PushDirection {
    Up,
    Left,
    Right,
    Down,
}

fn slice_rect(direction: PushDirection, rect: window::Rect, slice_factor: f32) -> window::Rect {
    let (width, height) = rect.wh();
    let width_slice = ((width as f32) / slice_factor) as i32;
    let height_slice = ((height as f32) / slice_factor) as i32;

    match direction {
        PushDirection::Up => window::Rect::xywh(rect.left, rect.top, width, height_slice),
        PushDirection::Left => window::Rect::xywh(rect.left, rect.top, width_slice, height),
        PushDirection::Right => window::Rect::xywh(
            rect.left + width - width_slice,
            rect.top,
            width_slice,
            height,
        ),
        PushDirection::Down => window::Rect::xywh(
            rect.left,
            rect.top + height - height_slice,
            width,
            height_slice,
        ),
    }
}

fn main() {
    pretty_env_logger::init();

    let mut kb = keybind::Keybinds::new();

    let mut make_push_kb = |key: i32, direction: PushDirection| {
        kb.register_keybind(
            key as u32,
            vec![
                keybind::Modifier::Alt,
                keybind::Modifier::Control,
                keybind::Modifier::Super,
            ],
            std::rc::Rc::new(move || {
                if let Some(mut active_window) = window::get_focused_window() {
                    let monitor = active_window.get_monitor();
                    let monitor_work_area = monitor.get_work_area().unwrap();
                    let pushed_rect = slice_rect(direction, monitor_work_area, 2.0);
                    active_window.set_rect(pushed_rect).unwrap();
                }
            }),
        )
        .unwrap();
    };

    make_push_kb(winapi::um::winuser::VK_UP, PushDirection::Up);
    make_push_kb(winapi::um::winuser::VK_LEFT, PushDirection::Left);
    make_push_kb(winapi::um::winuser::VK_RIGHT, PushDirection::Right);
    make_push_kb(winapi::um::winuser::VK_DOWN, PushDirection::Down);

    kb.start_message_loop().unwrap();
}
