// util before all others due to the macros it contains
mod util;

mod keybind;
mod window;

fn main() {
    pretty_env_logger::init();

    let mut kb = keybind::Keybinds::new();
    kb.register_keybind(
        winapi::um::winuser::VK_RIGHT as u32,
        vec![
            keybind::Modifier::Alt,
            keybind::Modifier::Control,
            keybind::Modifier::Super,
        ],
        std::rc::Rc::new(|| {
            if let Some(mut active_window) = window::get_focused_window() {
                let monitor = active_window.get_monitor();
                let monitor_work_area = monitor.get_work_area().unwrap();
                let (monitor_width, _monitor_height) = monitor_work_area.wh();

                let half_rect = window::Rect::xyxy(
                    monitor_work_area.left + monitor_width / 2,
                    monitor_work_area.top,
                    monitor_work_area.right + monitor_width,
                    monitor_work_area.bottom,
                );

                active_window.set_rect(half_rect).unwrap();
            }
        }),
    )
    .unwrap();

    kb.start_message_loop().unwrap();
}
