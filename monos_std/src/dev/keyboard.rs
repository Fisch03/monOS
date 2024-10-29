pub use pc_keyboard::{DecodedKey, KeyCode, KeyState};

use crate::messaging::{GenericMessage, MessageData, MessageType};

#[derive(Debug, Clone)]
pub struct Key {
    pub code: KeyCode,
    pub uppercase: bool,
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
            self.key.uppercase as u64,
            self.state as u64,
            0,
        )
    }

    unsafe fn from_message(message: GenericMessage) -> Option<Self> {
        let data = message.data.as_scalar()?;

        Some(KeyEvent {
            key: Key {
                code: core::mem::transmute(data.0 as u8),
                uppercase: data.1 != 0,
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
            _ => return None,
        };

        Some(if self.uppercase {
            lower.to_ascii_uppercase()
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

impl From<DecodedKey> for Key {
    fn from(key: DecodedKey) -> Self {
        match key {
            DecodedKey::Unicode(c) => Key {
                code: keycode_from_char(c).unwrap(),
                uppercase: c.is_uppercase(),
            },
            DecodedKey::RawKey(code) => Key {
                code,
                uppercase: false,
            },
        }
    }
}

fn keycode_from_char(c: char) -> Option<KeyCode> {
    Some(match c.to_ascii_lowercase() {
        'a' => KeyCode::A,
        'b' => KeyCode::B,
        'c' => KeyCode::C,
        'd' => KeyCode::D,
        'e' => KeyCode::E,
        'f' => KeyCode::F,
        'g' => KeyCode::G,
        'h' => KeyCode::H,
        'i' => KeyCode::I,
        'j' => KeyCode::J,
        'k' => KeyCode::K,
        'l' => KeyCode::L,
        'm' => KeyCode::M,
        'n' => KeyCode::N,
        'o' => KeyCode::O,
        'p' => KeyCode::P,
        'q' => KeyCode::Q,
        'r' => KeyCode::R,
        's' => KeyCode::S,
        't' => KeyCode::T,
        'u' => KeyCode::U,
        'v' => KeyCode::V,
        'w' => KeyCode::W,
        'x' => KeyCode::X,
        'y' => KeyCode::Y,
        'z' => KeyCode::Z,
        '1' => KeyCode::Key1,
        '2' => KeyCode::Key2,
        '3' => KeyCode::Key3,
        '4' => KeyCode::Key4,
        '5' => KeyCode::Key5,
        '6' => KeyCode::Key6,
        '7' => KeyCode::Key7,
        '8' => KeyCode::Key8,
        '9' => KeyCode::Key9,
        '0' => KeyCode::Key0,
        ' ' => KeyCode::Spacebar,
        _ => return None,
    })
}
