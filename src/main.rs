// util before all others due to the macros it contains
mod util;

mod cli;
mod config;
mod keybind;
mod window;

use structopt::StructOpt;

use winapi::shared::winerror;
use winapi::um::shellapi;
use winapi::um::winuser;

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
    let cli_options = cli::CliOptions::from_args();

    match cli_options.command {
        Some(cli::CliCommand::OpenConfigFile) => {
            let config_path = config::default_config_path();

            if !config_path.exists() {
                log::info!("Whimsy configuration file {} does not exist, writing a default configuration file to this location.", config_path.display());
                if let Err(error) = config::create_default_config() {
                    log::error!(
                        "Unable to write default configuration file to {}.\nError: {}",
                        config_path.display(),
                        error
                    );
                    std::process::exit(1);
                }
            }

            // Use Win32 to spawn the system's text editor.
            // EDITOR is not always set on Windows machines, so it's not the
            // right choice here.
            unsafe {
                use std::os::windows::prelude::*;
                let path_os_str = config_path.as_os_str();
                let mut path_bytes: Vec<u16> = path_os_str.encode_wide().collect();

                // The OsStr slice is not null-terminated, so we need to null-
                // terminate the byte sequence before we give it to Win32.
                path_bytes.push(0);

                let result_code = winapi::um::shellapi::ShellExecuteW(
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    path_bytes[..].as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    winuser::SW_SHOWNORMAL,
                    // Ugly: ShellExecute returns a fake HINSTANCE for backwards compatibility.
                    // It is actually a result code. We need to convert it to an i32 in order
                    // to actually check it.
                    // For more information on this mess, check the Win32 API documentation
                    // for ShellExecuteW:
                    // https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew#return-value
                ) as usize as i32;

                // A result code less than 32 indicates failure.
                if result_code <= 32 {
                    match result_code as u32 {
                        winerror::ERROR_FILE_NOT_FOUND => println!("Unable to open the configuration file at {}: file not found", config_path.display()),
                        shellapi::SE_ERR_ACCESSDENIED => println!("Unable to open the configuration file at {}: whimsy cannot access the configuration file (access denied).", config_path.display()),
                        _ => println!("Unable to open the configuration file at {} due to an internal OS error.\nError code: {}", config_path.display(), result_code),
                    }

                    std::process::exit(1);
                }
            }

            return;
        }
        None => {}
    }

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

    make_push_kb(winuser::VK_UP, PushDirection::Up);
    make_push_kb(winuser::VK_LEFT, PushDirection::Left);
    make_push_kb(winuser::VK_RIGHT, PushDirection::Right);
    make_push_kb(winuser::VK_DOWN, PushDirection::Down);

    kb.start_message_loop().unwrap();
}
