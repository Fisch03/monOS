use super::*;
use crate::gfx::*;

#[inline(always)]
pub fn open_fb() -> Option<Framebuffer> {
    let mut fb: Option<Framebuffer> = None;

    unsafe {
        syscall_1(
            Syscall::OpenFramebuffer,
            &mut fb as *mut Option<Framebuffer> as u64,
        )
    };

    fb
}

#[inline(always)]
pub fn submit_frame(framebuffer: &Framebuffer) {
    let frame = framebuffer.buffer();

    unsafe {
        syscall_2(
            Syscall::SubmitFrame,
            frame.as_ptr() as u64,
            frame.len() as u64,
        );
    }
}
