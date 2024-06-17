use super::*;
use crate::gfx::*;

#[inline(always)]
pub fn open_fb() -> Option<Framebuffer> {
    let mut fb_info: Option<FramebufferInfo> = None;

    unsafe {
        syscall_1(
            Syscall::OpenFramebuffer,
            &mut fb_info as *mut Option<FramebufferInfo> as u64,
        )
    };

    let fb_info = fb_info?;

    crate::println!("{:?}", fb_info);
    crate::println!(
        "{}",
        fb_info.dimensions.width * fb_info.dimensions.height * fb_info.bytes_per_pixel,
    );

    Some(Framebuffer::new(fb_info))
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
