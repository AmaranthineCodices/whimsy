use std::collections::HashMap;
use std::rc::Rc;

use winapi::um::winuser;

#[derive(Debug, Copy, Clone)]
pub enum Modifier {
    Control,
    Alt,
    Shift,
    Super,
}

fn modifier_to_flag_code(modifier: &Modifier) -> isize {
    match modifier {
        Modifier::Control => winuser::MOD_CONTROL,
        Modifier::Alt => winuser::MOD_ALT,
        Modifier::Shift => winuser::MOD_SHIFT,
        Modifier::Super => winuser::MOD_WIN,
    }
}

pub struct Keybinds {
    bind_id_increment: i32,
    active_binds: HashMap<i32, Rc<dyn FnMut() -> ()>>,
}

impl Keybinds {
    pub fn new() -> Keybinds {
        Keybinds {
            bind_id_increment: 0,
            active_binds: HashMap::new(),
        }
    }

    // FIXME: Use an actual enum instead of an integer code for better error checking.
    // Need to figure out how to autogenerate this.
    pub fn register_keybind(
        &mut self,
        key_code: u32,
        modifiers: Vec<Modifier>,
        callback: Rc<dyn FnMut() -> ()>,
    ) -> Result<(), ()> {
        let id = self.bind_id_increment;
        let modifier_flags = modifiers.iter().fold(0, |accumulator, modifier| {
            accumulator | modifier_to_flag_code(modifier)
        }) | winuser::MOD_NOREPEAT;

        unsafe {
            evaluate_fallible_winapi!(winuser::RegisterHotKey(
                std::ptr::null_mut(),
                id,
                // cast is safe, the maximum value of modifier_flags is 0x400F
                modifier_flags as u32,
                key_code
            ));
        }

        log::debug!(
            "Registered keybind with internal ID {:?}, modifiers {:?}, and virtual keycode {:?}.",
            id,
            modifiers,
            key_code
        );

        self.active_binds.insert(id, callback);
        self.bind_id_increment += 1;
        Ok(())
    }

    pub fn start_message_loop(mut self) -> Result<(), ()> {
        log::debug!("Starting keybind message loop.");

        unsafe {
            let mut msg: winuser::MSG = std::mem::zeroed();

            loop {
                // Do not use evaluate_fallible_winapi! because GetMessage has different return values than it can accept.
                let result = winuser::GetMessageW(
                    &mut msg,
                    std::ptr::null_mut(),
                    winuser::WM_HOTKEY,
                    winuser::WM_HOTKEY,
                );

                match result {
                    // Result code 0 is a WM_QUIT message; we should stop the loop here.
                    0 => break,
                    // -1 is an error.
                    -1 => {
                        log::error!(
                            "Error from GetMessageW: {}",
                            winapi::um::errhandlingapi::GetLastError(),
                        );
                        return Err(());
                    }
                    // Anything else is a successful message retrieval; if this is the case, `msg`
                    // is safe to read.
                    _ => {
                        debug_assert!(msg.message == winuser::WM_HOTKEY, "The keybind message loop only handles WM_HOTKEY messages, but it has received a message that is not a hotkey message.");
                        let id = msg.wParam as i32;

                        if !self.active_binds.contains_key(&id) {
                            log::error!("Unregistered keybind with ID {} was fired.", id);
                            continue;
                        }

                        log::trace!("Keybind {} pressed", id);
                        Rc::get_mut(&mut self.active_binds.get_mut(&id).unwrap()).unwrap()();
                    }
                }
            }
        }

        Ok(())
    }
}
