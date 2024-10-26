#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::{Color, Dimension};
use rooftop::{WindowClientMessage, WindowServerMessage};

#[no_mangle]
fn main() {
    println!("terminal started!");

    let win_channel = syscall::connect("desktop.windows").unwrap();
    win_channel.send(WindowClientMessage::CreateWindow {
        dimensions: Dimension::new(640 / 2, 480 / 2),
    });

    loop {
        while let Some(msg) = syscall::receive_any() {
            if msg.sender == win_channel {
                let msg = unsafe { WindowServerMessage::from_message(msg) };
                match msg {
                    Some(WindowServerMessage::RequestRender(mut chunk)) => {
                        chunk.set_title("terminal");

                        let mut fb = chunk.fb();
                        fb.clear();

                        win_channel.send(WindowClientMessage::SubmitRender(chunk));
                    }
                    _ => {
                        panic!("terminal: unexpected message: {:?}", msg);
                    }
                }
            }
        }

        syscall::yield_();
    }
}
