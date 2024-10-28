#![no_std]
#![feature(prelude_import)]

#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

mod windowing;
pub use windowing::{client::*, WindowClientMessage, WindowServerMessage};
