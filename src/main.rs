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
                active_window
                    .set_rect(window::Rect::xywh(0, 0, 800, 400))
                    .unwrap();
            }
        }),
    )
    .unwrap();

    kb.start_message_loop().unwrap();
}
