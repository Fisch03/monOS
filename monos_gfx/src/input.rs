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
    pub fn update(&mut self, state: MouseState, bounds: Rect) {
        self.position.x += state.x as i64;
        self.position.x = self.position.x.max(bounds.min.x).min(bounds.max.x);
        self.position.y -= state.y as i64;
        self.position.y = self.position.y.max(bounds.min.y).min(bounds.max.y);
        self.left_button.clicked = state.flags.left_button();
        self.right_button.clicked = state.flags.right_button();
        self.middle_button.clicked = state.flags.middle_button();
    }
}

#[derive(Debug, Clone, Default)]
pub struct MouseButtonState {
    pub clicked: bool,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub struct KeyboardState {
    //TODO
}
