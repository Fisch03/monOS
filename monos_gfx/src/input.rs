use alloc::collections::VecDeque;

pub struct MousePacket {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

pub struct InputBuffers {
    //keyboard: VecDeque<u8>,
    mouse: VecDeque<u8>,
}

impl InputBuffers {
    pub fn new() -> Self {
        Self {
            //keyboard: VecDeque::new(),
            mouse: VecDeque::new(),
        }
    }
}
