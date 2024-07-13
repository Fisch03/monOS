use crate::types::*;

use alloc::collections::VecDeque;
pub use pc_keyboard::{DecodedKey as Key, KeyCode as RawKey, KeyState};

pub use monos_std::dev::mouse::{MouseFlags, MouseState};

#[derive(Debug, Default, Clone)]
pub struct Input {
    pub mouse: MouseInput,

    pub keyboard: VecDeque<KeyEvent>,
}

impl Input {
    pub fn clear(&mut self) {
        self.mouse.clear();
        self.keyboard.clear();
    }
}

#[derive(Debug, Clone, Default)]
pub struct MouseInput {
    pub position: Position,
    pub scroll: i64,

    pub left_button: MouseButtonState,
    pub right_button: MouseButtonState,
    pub middle_button: MouseButtonState,
}

impl MouseInput {
    pub fn update_new(&mut self, state: MouseState, bounds: Rect) {
        self.position.x += state.x as i64;
        self.position.x = self.position.x.max(bounds.min.x).min(bounds.max.x);
        self.position.y -= state.y as i64;
        self.position.y = self.position.y.max(bounds.min.y).min(bounds.max.y);
        self.scroll += state.scroll as i64;
        self.left_button.update(state.flags.left_button());
        self.right_button.update(state.flags.right_button());
        self.middle_button.update(state.flags.middle_button());
    }

    pub fn clear(&mut self) {
        self.left_button.clicked = false;
        self.right_button.clicked = false;
        self.middle_button.clicked = false;
        self.scroll = 0;
    }
}

#[derive(Debug, Clone, Default)]
pub struct MouseButtonState {
    pub clicked: bool,
    pub pressed: bool,
}

impl MouseButtonState {
    pub fn update(&mut self, state: bool) {
        self.clicked = !self.pressed && state;
        self.pressed = state;
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub state: KeyState,
}
