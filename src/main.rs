// util before all others due to the macros it contains
mod util;

mod cli;
mod config;
mod keybind;
mod window;

use std::collections::HashMap;

use color_eyre::eyre::Result;

use structopt::StructOpt;

use winapi::shared::winerror;
use winapi::um::shellapi;
use winapi::um::winuser;

fn slice_rect(direction: config::Direction, rect: window::Rect, slice_factor: f32) -> window::Rect {
    let (width, height) = rect.wh();
    let width_slice = ((width as f32) / slice_factor) as i32;
    let height_slice = ((height as f32) / slice_factor) as i32;

    match direction {
        config::Direction::Up => window::Rect::xywh(rect.left, rect.top, width, height_slice),
        config::Direction::Left => window::Rect::xywh(rect.left, rect.top, width_slice, height),
        config::Direction::Right => window::Rect::xywh(
            rect.left + width - width_slice,
            rect.top,
            width_slice,
            height,
        ),
        config::Direction::Down => window::Rect::xywh(
            rect.left,
            rect.top + height - height_slice,
            width,
            height_slice,
        ),
    }
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    color_eyre::install()?;

    let cli_options = cli::CliOptions::from_args();
    match cli_options.command {
        Some(cli::CliCommand::OpenConfigFile) => {
            let config_path = &config::DEFAULT_CONFIG_PATH;

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

            return Ok(());
        }
        Some(cli::CliCommand::RegenerateConfigFile) => {
            config::create_default_config()?;

            return Ok(());
        }
        None => {}
    }

    let config_path = cli_options.config_file;

    let config = config::read_config_from_file(&config_path)?.unwrap_or_default();
    let mut kb = keybind::Keybinds::new();
    let mut kb_bindings = HashMap::new();

    for binding in &config.bindings {
        let binding_id = kb
            .register_keybind(binding.key, &binding.modifiers)
            .unwrap();
        kb_bindings.insert(binding_id, binding);
    }

    loop {
        match kb.poll_message_loop().unwrap() {
            keybind::KeybindMessage::Quit => {
                log::debug!("Stopping keybind message polling due to a quit message");
                break;
            }
            keybind::KeybindMessage::BindActivated(id) => {
                let &binding = kb_bindings.get(&id).unwrap();

                match binding.action {
                    config::Action::Push {
                        direction,
                        fraction,
                    } => {
                        if let Some(mut active_window) = window::get_focused_window() {
                            let monitor = active_window.get_monitor();
                            let monitor_work_area = monitor.get_work_area().unwrap();
                            let pushed_rect = slice_rect(direction, monitor_work_area, fraction);
                            log::debug!(
                                "Pushed active window (direction {:?}, fraction {:?}) to rect {:?}",
                                direction,
                                fraction,
                                pushed_rect
                            );
                            active_window.set_rect(pushed_rect).unwrap();
                        }
                    }
                    config::Action::Nudge {
                        direction,
                        distance,
                    } => {
                        if let Some(mut active_window) = window::get_focused_window() {
                            let starting_rect = active_window.get_rect().unwrap();
                            let (width, height) = starting_rect.wh();
                            let absolute_distance = match distance {
                                config::Metric::Absolute(value) => value,
                                config::Metric::Percent(fraction) => match direction {
                                    config::Direction::Up | config::Direction::Down => {
                                        height as f32 * fraction
                                    }
                                    config::Direction::Left | config::Direction::Right => {
                                        width as f32 * fraction
                                    }
                                },
                            } as i32;

                            let nudged_rect = match direction {
                                config::Direction::Up => window::Rect::xywh(
                                    starting_rect.left,
                                    starting_rect.top - absolute_distance,
                                    width,
                                    height,
                                ),
                                config::Direction::Down => window::Rect::xywh(
                                    starting_rect.left,
                                    starting_rect.top + absolute_distance,
                                    width,
                                    height,
                                ),
                                config::Direction::Left => window::Rect::xywh(
                                    starting_rect.left - absolute_distance,
                                    starting_rect.top,
                                    width,
                                    height,
                                ),
                                config::Direction::Right => window::Rect::xywh(
                                    starting_rect.left + absolute_distance,
                                    starting_rect.top,
                                    width,
                                    height,
                                ),
                            };

                            active_window.set_rect(nudged_rect).unwrap();
                            log::debug!(
                                "Nudged active window {:?}px in direction {:?} - new rect {:?}",
                                absolute_distance,
                                direction,
                                nudged_rect
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
