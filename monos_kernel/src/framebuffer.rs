use crate::mem::{self, VirtualAddress};

use bootloader_api::info::FrameBuffer as RawFrameBuffer;
use core::slice;
use spin::{Mutex, MutexGuard, Once};

use monos_gfx::{types::*, Framebuffer, FramebufferInfo};

static FRAMEBUFFER: Once<Mutex<KernelFramebuffer>> = Once::new();

pub fn init(fb: RawFrameBuffer) {
    FRAMEBUFFER.call_once(|| Mutex::new(KernelFramebuffer::new(fb)));
}

pub fn get() -> Option<MutexGuard<'static, KernelFramebuffer>> {
    FRAMEBUFFER.get().map(|fb| fb.lock())
}

pub struct KernelFramebuffer {
    front_buffer: &'static mut [u8],
    framebuffer: Framebuffer,
    borrowed: Option<usize>,
}

impl KernelFramebuffer {
    fn new(fb: RawFrameBuffer) -> Self {
        let info = fb.info();

        let front_buffer = fb.into_buffer();
        let front_buffer_virt = VirtualAddress::from_ptr(front_buffer);
        let front_buffer_phys = mem::translate_addr(front_buffer_virt).unwrap();

        let mut front_buffer_page = mem::Page::around(front_buffer_virt);
        let mut front_buffer_frame = mem::Frame::around(front_buffer_phys);
        let front_buffer_end_page =
            mem::Page::around(front_buffer_virt + info.byte_len as u64).next();

        use mem::PageTableFlags;
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::WRITE_THROUGH
            | PageTableFlags::CACHE_DISABLE;

        loop {
            unsafe { mem::map_to(&front_buffer_page, &front_buffer_frame, flags) }
                .expect("failed to map frame buffer");

            if front_buffer_page == front_buffer_end_page {
                break;
            }

            front_buffer_frame = front_buffer_frame.next();
            front_buffer_page = front_buffer_page.next();
        }

        crate::dbg!(info.byte_len);

        let dimensions = Dimension::new(info.width, info.height);
        let info = FramebufferInfo {
            dimensions,
            stride: info.stride as usize,
            bytes_per_pixel: info.bytes_per_pixel as usize,
        };

        let framebuffer = Self {
            front_buffer,
            framebuffer: Framebuffer::new(info),
            borrowed: None,
        };

        framebuffer
    }

    #[inline]
    pub fn submit_frame(&mut self, frame: &[u8]) {
        assert!(frame.len() == self.front_buffer.len());
        unsafe {
            core::ptr::copy_nonoverlapping(
                frame.as_ptr(),
                self.front_buffer.as_mut_ptr(),
                self.front_buffer.len(),
            );
        }
    }

    #[inline]
    pub fn submit_kernel_frame(&mut self) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.framebuffer.buffer().as_ptr(),
                self.front_buffer.as_mut_ptr(),
                self.front_buffer.len(),
            );
        }
    }

    pub fn borrow(&mut self, receiver: &mut Option<FramebufferInfo>, process_id: usize) {
        if self.borrowed.is_some() {
            receiver.take();
        } else {
            self.borrowed = Some(process_id);

            receiver.replace(*self.framebuffer.info());
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut Framebuffer> {
        if self.borrowed.is_some() {
            None
        } else {
            Some(&mut self.framebuffer)
        }
    }

    /// borrows the framebuffer mutably, even if it may lead to undefined behavior.
    ///
    ///  this is intended as a last resort in kernel panics, where we want to print the panic message to the framebuffer.
    pub unsafe fn now_or_never(&mut self) -> &mut Framebuffer {
        &mut self.framebuffer
    }
}
