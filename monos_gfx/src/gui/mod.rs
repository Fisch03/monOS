use crate::Dimension;
use alloc::vec::Vec;

pub struct GUIFrame {
    dimensions: Dimension,

    buffer: Vec<u8>,
}
