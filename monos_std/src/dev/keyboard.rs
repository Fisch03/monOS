pub use pc_keyboard::{DecodedKey, KeyCode, KeyState};

use crate::messaging::{GenericMessage, MessageData, MessageType};

pub const MODIFIER_SHIFT: u8 = 0b0000_0001;
pub const MODIFIER_CTRL: u8 = 0b0000_0010;
pub const MODIFIER_ALT: u8 = 0b0000_0100;
pub const MODIFIER_GUI: u8 = 0b0000_1000;

#[derive(Debug, Clone)]
pub struct Key {
    pub code: KeyCode,
    modifiers: u8,
}

impl Key {
    pub fn new(code: KeyCode, shift: bool, ctrl: bool, alt: bool, gui: bool) -> Self {
        let mut modifiers = 0;
        if shift {
            modifiers |= MODIFIER_SHIFT;
        }
        if ctrl {
            modifiers |= MODIFIER_CTRL;
        }
        if alt {
            modifiers |= MODIFIER_ALT;
        }
        if gui {
            modifiers |= MODIFIER_GUI;
        }

        Key { code, modifiers }
    }
    pub fn shift(&self) -> bool {
        self.modifiers & MODIFIER_SHIFT != 0
    }
    pub fn ctrl(&self) -> bool {
        self.modifiers & MODIFIER_CTRL != 0
    }
    pub fn alt(&self) -> bool {
        self.modifiers & MODIFIER_ALT != 0
    }
    pub fn gui(&self) -> bool {
        self.modifiers & MODIFIER_GUI != 0
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub state: KeyState,
}

impl MessageData for KeyEvent {
    fn into_message(self) -> MessageType {
        MessageType::Scalar(
            self.key.code as u64,
            self.key.modifiers as u64,
            self.state as u64,
            0,
        )
    }

    unsafe fn from_message(message: GenericMessage) -> Option<Self> {
        let data = message.data.as_scalar()?;

        Some(KeyEvent {
            key: Key {
                code: core::mem::transmute(data.0 as u8),
                modifiers: data.1 as u8,
            },
            state: unsafe { core::mem::transmute(data.2 as u8) },
        })
    }
}

impl Key {
    pub fn as_char(&self) -> Option<char> {
        let lower = match self.code {
            KeyCode::A => 'a',
            KeyCode::B => 'b',
            KeyCode::C => 'c',
            KeyCode::D => 'd',
            KeyCode::E => 'e',
            KeyCode::F => 'f',
            KeyCode::G => 'g',
            KeyCode::H => 'h',
            KeyCode::I => 'i',
            KeyCode::J => 'j',
            KeyCode::K => 'k',
            KeyCode::L => 'l',
            KeyCode::M => 'm',
            KeyCode::N => 'n',
            KeyCode::O => 'o',
            KeyCode::P => 'p',
            KeyCode::Q => 'q',
            KeyCode::R => 'r',
            KeyCode::S => 's',
            KeyCode::T => 't',
            KeyCode::U => 'u',
            KeyCode::V => 'v',
            KeyCode::W => 'w',
            KeyCode::X => 'x',
            KeyCode::Y => 'y',
            KeyCode::Z => 'z',
            KeyCode::Key1 => '1',
            KeyCode::Key2 => '2',
            KeyCode::Key3 => '3',
            KeyCode::Key4 => '4',
            KeyCode::Key5 => '5',
            KeyCode::Key6 => '6',
            KeyCode::Key7 => '7',
            KeyCode::Key8 => '8',
            KeyCode::Key9 => '9',
            KeyCode::Key0 => '0',
            KeyCode::Spacebar => ' ',
            KeyCode::OemMinus => '-',
            KeyCode::OemPlus => '=',
            KeyCode::OemComma => ',',
            KeyCode::OemPeriod => '.',
            KeyCode::Oem1 => ';',
            KeyCode::Oem2 => '/',
            KeyCode::Oem3 => '`',
            KeyCode::Oem4 => '[',
            KeyCode::Oem6 => ']',
            KeyCode::Oem5 => '\\',
            KeyCode::Oem7 => '\'',
            _ => return None,
        };

        Some(if self.shift() {
            match lower {
                '1' => '!',
                '2' => '@',
                '3' => '#',
                '4' => '$',
                '5' => '%',
                '6' => '^',
                '7' => '&',
                '8' => '*',
                '9' => '(',
                '0' => ')',
                '-' => '_',
                '=' => '+',
                ',' => '<',
                '.' => '>',
                ';' => ':',
                '/' => '?',
                '[' => '{',
                ']' => '}',
                '\\' => '|',
                '\'' => '"',
                _ => lower.to_ascii_uppercase(),
            }
        } else {
            lower
        })
    }
}

impl AsRef<KeyCode> for Key {
    fn as_ref(&self) -> &KeyCode {
        &self.code
    }
}
