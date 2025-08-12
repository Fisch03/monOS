#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]
#![feature(c_variadic)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

mod libc;

extern crate alloc;

use alloc::ffi::CString;
use monos_gfx::{
    input::{KeyCode, KeyEvent, KeyState},
    Dimension, Framebuffer, FramebufferFormat, Input,
};
use monos_std::collections::VecDeque;
use rooftop::{Window, WindowClient};

use core::sync::atomic::{AtomicBool, Ordering};

static FRAME_READY: AtomicBool = AtomicBool::new(false);
static mut KEY_QUEUE: VecDeque<KeyEvent> = VecDeque::new();

extern "C" {
    static DG_ScreenBuffer: *mut u8;
    fn doomgeneric_Create(argc: i32, argv: *const *const u8);
    fn doomgeneric_Tick();
    fn DG_AddMouse(delta_x: i32, delta_y: i32, buttons: i32);
}

#[no_mangle]
pub unsafe extern "C" fn DG_Init() {}
#[no_mangle]
pub unsafe extern "C" fn DG_DrawFrame() {
    FRAME_READY.store(true, Ordering::Relaxed);
}

#[no_mangle]
pub unsafe extern "C" fn DG_SleepMs(ms: u32) {
    let target = syscall::get_time() + ms as u64;
    loop {
        if syscall::get_time() >= target {
            break;
        }

        syscall::yield_();
    }
}

#[no_mangle]
pub unsafe extern "C" fn DG_GetTicksMs() -> u32 {
    syscall::get_time() as u32
}

#[no_mangle]
pub unsafe extern "C" fn DG_GetKey(pressed: *mut i32, doom_key: *mut u8) -> i32 {
    const DOOM_KEY_RIGHTARROW: u8 = 0xae;
    const DOOM_KEY_LEFTARROW: u8 = 0xac;
    const DOOM_KEY_UPARROW: u8 = 0xad;
    const DOOM_KEY_DOWNARROW: u8 = 0xaf;
    const DOOM_KEY_STRAFE_L: u8 = 0xa0;
    const DOOM_KEY_STRAFE_R: u8 = 0xa1;
    const DOOM_KEY_USE: u8 = 0xa2;
    const DOOM_KEY_FIRE: u8 = 0xa3;
    const DOOM_KEY_ESCAPE: u8 = 27;
    const DOOM_KEY_ENTER: u8 = 13;

    if let Some(evt) = unsafe { KEY_QUEUE.pop_front() } {
        unsafe {
            *doom_key = match evt.key.code {
                KeyCode::W | KeyCode::ArrowUp => DOOM_KEY_UPARROW,
                KeyCode::A => DOOM_KEY_STRAFE_L,
                KeyCode::S | KeyCode::ArrowDown => DOOM_KEY_DOWNARROW,
                KeyCode::D => DOOM_KEY_STRAFE_R,
                KeyCode::E => DOOM_KEY_USE,
                KeyCode::ArrowLeft => DOOM_KEY_LEFTARROW,
                KeyCode::ArrowRight => DOOM_KEY_RIGHTARROW,

                KeyCode::LControl => DOOM_KEY_FIRE,
                KeyCode::Return => DOOM_KEY_ENTER,
                KeyCode::Escape => DOOM_KEY_ESCAPE,
                _ => return 0,
            };
        }

        unsafe {
            *pressed = match evt.state {
                KeyState::Down => 1,
                KeyState::Up => 0,
                KeyState::SingleShot => 0,
            };
        }

        1
    } else {
        0
    }
}
#[no_mangle]
pub unsafe extern "C" fn DG_SetWindowTitle(title: *const i8) {
    let title = core::ffi::CStr::from_ptr(title);
    println!("window title: {}", title.to_str().unwrap());
}

#[no_mangle]
fn main() {
    let mut window_client = WindowClient::new("desktop.windows", ()).unwrap();
    window_client.create_window("doom", Dimension::new(320, 200), render);

    let wad = if args().len() > 0 {
        let slice = args()[0].as_str();
        CString::new(slice).expect("invalid wad path")
    } else {
        CString::new("data/wads/doom1.wad").unwrap()
    };

    let args: [&core::ffi::CStr; 4] = [c"bin/doom", c"-iwad", &wad, c"-nosound"];
    let args = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    unsafe { doomgeneric_Create(args.len() as i32, args.as_ptr() as *const *const u8) };

    loop {
        unsafe {
            doomgeneric_Tick();
        }

        window_client.update();

        syscall::yield_();
    }
}

fn render(window: &mut Window, _app: &mut (), input: Input) {
    *window.update_frequency = rooftop::UpdateFrequency::Always;
    *window.grab_mouse = true;

    input.keyboard.keys.iter().for_each(|evt| unsafe {
        KEY_QUEUE.push_back(evt.clone());
    });

    if window.mouse_grabbed {
        unsafe {
            DG_AddMouse(
                input.mouse.delta.x as i32 * 5,
                -input.mouse.delta.y as i32 * 5,
                if input.mouse.left_button.pressed {
                    1
                } else {
                    0
                },
            );
        }
    }

    if !FRAME_READY.swap(false, Ordering::Relaxed) {
        // this might introduce some tearing but it improves the framerate so whatevs
        // return;
    }

    let doom_fb = unsafe { core::slice::from_raw_parts_mut(DG_ScreenBuffer, 320 * 200 * 3) };
    let doom_fb = Framebuffer::new(
        doom_fb,
        Dimension::new(320, 200),
        FramebufferFormat {
            bytes_per_pixel: 3,
            stride: 320,
            r_position: 0,
            g_position: 1,
            b_position: 2,
            a_position: None,
        },
    );

    window.clear_with(&doom_fb);
}
