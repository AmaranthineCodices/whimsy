use std::collections::HashMap;
use std::rc::Rc;

use winapi::um::winuser;

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Modifier {
    #[serde(alias = "ctrl")]
    Control,
    Alt,
    Shift,
    #[serde(alias = "win")]
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

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[repr(i32)]
#[serde(rename_all = "kebab-case")]
pub enum Key {
    Backspace = 0x08,
    Tab = 0x09,
    Clear = 0x0C,
    #[serde(alias = "return")]
    Enter = 0x0D,
    Pause = 0x13,
    CapsLock = 0x14,
    Escape = 0x1B,
    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
    Select = 0x29,
    PrintScreen = 0x2C,
    Insert = 0x2D,
    Delete = 0x2E,
    #[serde(alias = "0")]
    Zero = 0x30,
    #[serde(alias = "1")]
    One = 0x31,
    #[serde(alias = "2")]
    Two = 0x32,
    #[serde(alias = "3")]
    Three = 0x33,
    #[serde(alias = "4")]
    Four = 0x34,
    #[serde(alias = "5")]
    Five = 0x35,
    #[serde(alias = "6")]
    Six = 0x36,
    #[serde(alias = "7")]
    Seven = 0x37,
    #[serde(alias = "8")]
    Eight = 0x38,
    #[serde(alias = "9")]
    Nine = 0x39,
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,
    #[serde(alias = "numpad0")]
    NumpadZero = 0x60,
    #[serde(alias = "numpad1")]
    NumpadOne = 0x61,
    #[serde(alias = "numpad2")]
    NumpadTwo = 0x62,
    #[serde(alias = "numpad3")]
    NumpadThree = 0x63,
    #[serde(alias = "numpad4")]
    NumpadFour = 0x64,
    #[serde(alias = "numpad5")]
    NumpadFive = 0x65,
    #[serde(alias = "numpad6")]
    NumpadSix = 0x66,
    #[serde(alias = "numpad7")]
    NumpadSeven = 0x67,
    #[serde(alias = "numpad8")]
    NumpadEight = 0x68,
    #[serde(alias = "numpad9")]
    NumpadNine = 0x69,
    #[serde(alias = "star")]
    Multiply = 0x6A,
    #[serde(alias = "add")]
    Plus = 0x6B,
    #[serde(alias = "subtract")]
    Minus = 0x6D,
    #[serde(alias = "period")]
    Decimal = 0x6E,
    Divide = 0x6F,
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,
    NumLock = 0x90,
    ScrollLock = 0x91,
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
