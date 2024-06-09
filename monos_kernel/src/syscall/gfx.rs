use monos_gfx::OpenedFramebuffer;

use super::LOWER_HALF_END;

pub fn sys_open_fb(arg1: u64) {
    assert!(arg1 <= LOWER_HALF_END);

    let fb_ptr = arg1 as *mut Option<OpenedFramebuffer>;
    let mut fb = unsafe { &mut *fb_ptr };

    if let Some(mut fb_guard) = crate::framebuffer::get() {
        fb_guard.borrow(&mut fb)
    }
}
