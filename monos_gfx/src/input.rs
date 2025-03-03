use crate::types::*;

pub use monos_std::dev::keyboard::{Key, KeyCode, KeyEvent, KeyState};
use monos_std::dev::mouse::MouseState;

#[derive(Debug, Default, Clone)]
pub struct Input {
    pub mouse: MouseInput,
    pub keyboard: KeyboardInput,
}

impl Input {
    pub fn clear(&mut self) {
        self.mouse.clear();
        self.keyboard.clear();
    }

    pub fn any(&self) -> bool {
        self.mouse.left_button.clicked
            || self.mouse.right_button.clicked
            || self.mouse.middle_button.clicked
            || self.mouse.scroll != 0
            || !self.keyboard.keys.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MouseInput {
    pub position: Position,
    pub delta: Position,
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

        self.delta.x += state.x as i64;
        self.delta.y -= state.y as i64;

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
        self.delta = Position::zero();
    }

    pub fn moved(&self) -> bool {
        self.delta.x != 0 || self.delta.y != 0
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

#[derive(Debug, Clone, Default)]
pub struct KeyboardInput {
    pub keys: Vec<KeyEvent>,
}

impl KeyboardInput {
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    pub fn pressed(&self, key: KeyCode) -> bool {
        self.keys
            .iter()
            .any(|e| e.key.code == key && e.state == KeyState::Down)
    }

    pub fn consume(&mut self) -> Option<KeyEvent> {
        self.keys.pop()
    }

    pub fn consume_key(&mut self, key: KeyCode) -> Option<KeyState> {
        let index = self.keys.iter().position(|e| e.key.code == key);
        if let Some(index) = index {
            Some(self.keys.remove(index).state)
        } else {
            None
        }
    }
}
