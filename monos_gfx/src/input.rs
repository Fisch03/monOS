use crate::types::*;

pub struct Input {
    pub mouse: MouseInput,
    pub keyboard: KeyboardState,
}

#[derive(Debug, Clone)]
pub struct MouseInput {
    pub position: Position,

    pub left_button: MouseButtonState,
    pub right_button: MouseButtonState,
    pub middle_button: MouseButtonState,
}

#[derive(Debug, Clone)]
pub struct MouseButtonState {
    pub clicked: bool,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub struct KeyboardState {
    //TODO
}
