use crate::types::*;
use monos_std::dev::mouse::MouseState;

#[derive(Debug, Default)]
pub struct Input {
    pub mouse: MouseInput,
    // pub keyboard: KeyboardState,
}

#[derive(Debug, Clone, Default)]
pub struct MouseInput {
    pub position: Position,

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
        self.left_button.update(state.flags.left_button());
        self.right_button.update(state.flags.right_button());
        self.middle_button.update(state.flags.middle_button());
    }

    pub fn update(&mut self) {
        self.left_button.clicked = false;
        self.right_button.clicked = false;
        self.middle_button.clicked = false;
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
pub struct KeyboardState {
    //TODO
}
