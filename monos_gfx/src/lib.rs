#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

extern crate alloc;

pub mod types;
pub use types::*;

pub mod text;
pub use text::font;

pub mod image;
pub use image::{Image, ImageFormat};

pub mod input;
pub use input::Input;

pub mod ui;

#[cfg(feature = "userspace")]
pub mod paint;
#[cfg(feature = "userspace")]
pub use paint::PaintFramebuffer;

pub mod framebuffer;
pub use framebuffer::{Framebuffer, FramebufferFormat};
