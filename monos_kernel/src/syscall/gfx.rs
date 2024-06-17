use crate::process;
use monos_gfx::FramebufferInfo;

use super::LOWER_HALF_END;

pub fn sys_open_fb(arg1: u64) {
    assert!(arg1 <= LOWER_HALF_END);

    let fb_ptr = arg1 as *mut Option<FramebufferInfo>;
    let mut fb = unsafe { &mut *fb_ptr };

    if let Some(mut fb_guard) = crate::framebuffer::get() {
        let process_id = process::current_pid().unwrap();

        fb_guard.borrow(&mut fb, process_id);
    }
}

pub fn sys_submit_frame(arg1: u64, arg2: u64) {
    assert!(arg1 <= LOWER_HALF_END);
    let framebuffer = unsafe { core::slice::from_raw_parts(arg1 as *const u8, arg2 as usize) };
    if let Some(mut fb_guard) = crate::framebuffer::get() {
        fb_guard.submit_frame(framebuffer);
    }
}
