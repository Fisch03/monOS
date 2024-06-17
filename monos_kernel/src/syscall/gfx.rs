use crate::{mem::VirtualAddress, process};
use monos_gfx::*;

use super::LOWER_HALF_END;

pub fn sys_open_fb(arg1: u64) {
    assert!(arg1 <= LOWER_HALF_END);

    let fb_ptr = arg1 as *mut Option<Framebuffer>;
    let mut fb = unsafe { &mut *fb_ptr };

    if let Some(mut fb_guard) = crate::framebuffer::get() {
        let mut current_proc = process::CURRENT_PROCESS.write();
        let current_proc = current_proc.as_mut().unwrap();

        fb_guard.borrow(
            current_proc.id(),
            &mut fb,
            current_proc.mapper(),
            VirtualAddress::new(0x410000000000),
        );
    }
}

pub fn sys_submit_frame(arg1: u64, arg2: u64) {
    assert!(arg1 <= LOWER_HALF_END);
    let framebuffer = unsafe { core::slice::from_raw_parts(arg1 as *const u8, arg2 as usize) };
    if let Some(mut fb_guard) = crate::framebuffer::get() {
        fb_guard.submit_frame(framebuffer);
    }
}
