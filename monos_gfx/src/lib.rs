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

pub mod fonts;
pub mod image;
pub use image::{Image, ImageFormat};

pub mod input;
pub mod ui;

pub mod paint;

pub mod framebuffer;
pub use framebuffer::{Framebuffer, FramebufferFormat};
