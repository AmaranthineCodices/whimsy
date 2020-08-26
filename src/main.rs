// util before all others due to the macros it contains
mod util;

mod keybind;
mod window;

fn main() {
    pretty_env_logger::init();

    let mut kb = keybind::Keybinds::new();
    kb.register_keybind(
        winapi::um::winuser::VK_F1 as u32,
        vec![
            keybind::Modifier::Shift,
            keybind::Modifier::Alt,
            keybind::Modifier::Control,
            keybind::Modifier::Super,
        ],
        std::rc::Rc::new(|| {
            println!("Ctrl+Shift+Alt+Super+F1 pressed!");
        }),
    )
    .unwrap();

    kb.start_message_loop().unwrap();
}
