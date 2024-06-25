#![no_std]
#![no_main]

extern crate alloc;

pub mod types;
pub use types::*;

pub mod fonts;
pub mod input;
pub mod ui;

pub mod framebuffer;
pub use framebuffer::Framebuffer;
