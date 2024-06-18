#![no_std]
#![no_main]

extern crate alloc;

pub mod types;
pub use types::*;

pub mod gui;

mod fonts;

mod framebuffer;
pub use framebuffer::Framebuffer;
