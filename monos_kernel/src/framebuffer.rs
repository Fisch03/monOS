use crate::mem::{self, VirtualAddress};

use bootloader_api::info::{FrameBuffer as RawFrameBuffer, FrameBufferInfo};
use core::slice;
use spin::{Mutex, MutexGuard, Once};

use monos_gfx::{types::*, OpenedFramebuffer};

static FRAMEBUFFER: Once<Mutex<Framebuffer>> = Once::new();

pub fn init(fb: RawFrameBuffer) {
    FRAMEBUFFER.call_once(|| Mutex::new(Framebuffer::new(fb)));
}

pub fn get() -> Option<MutexGuard<'static, Framebuffer>> {
    FRAMEBUFFER.get().map(|fb| fb.lock())
}

pub struct Framebuffer {
    fb: OpenedFramebuffer,
    borrowed: bool,
}

impl Framebuffer {
    fn new(fb: RawFrameBuffer) -> Self {
        let info = fb.info();

        let front_buffer = fb.into_buffer();
        let front_buffer_virt = VirtualAddress::from_ptr(front_buffer);
        let front_buffer_phys = mem::translate_addr(front_buffer_virt).unwrap();

        let mut front_buffer_page = mem::Page::around(front_buffer_virt);
        let mut front_buffer_frame = mem::Frame::around(front_buffer_phys);
        let front_buffer_end_page = mem::Page::around(front_buffer_virt + info.byte_len as u64);

        // let front_buffer = front_buffer_page.start_address().as_mut_ptr::<u8>();
        // let front_buffer = unsafe { slice::from_raw_parts_mut(front_buffer, info.byte_len) };

        let back_buffer_virt = mem::alloc_vmem(info.byte_len as u64);
        let back_buffer_phys = mem::translate_addr(back_buffer_virt).unwrap_or_else(|_| {
            mem::alloc_frame()
                .expect("failed to allocate frame for back buffer")
                .start_address()
        });
        let mut back_buffer_page = mem::Page::around(back_buffer_virt);
        let back_buffer_frame = mem::Frame::around(back_buffer_phys);

        let back_buffer = back_buffer_page.start_address().as_mut_ptr::<u8>();
        let back_buffer = unsafe { slice::from_raw_parts_mut(back_buffer, info.byte_len) };

        use mem::PageTableFlags;
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::WRITE_THROUGH
            | PageTableFlags::CACHE_DISABLE
            | PageTableFlags::USER_ACCESSIBLE; // TODO: instead map fb to user space memory

        loop {
            unsafe { mem::map_to(&front_buffer_page, &front_buffer_frame, flags) }
                .expect("failed to map frame buffer");

            unsafe { mem::map_to(&back_buffer_page, &back_buffer_frame, flags) }
                .expect("failed to map back buffer");

            if front_buffer_page == front_buffer_end_page {
                break;
            }

            front_buffer_frame = front_buffer_frame.next();
            front_buffer_page = front_buffer_page.next();
            back_buffer_page = back_buffer_page.next();
        }

        let dimensions = Dimension::new(info.width, info.height);
        let framebuffer = Self {
            fb: OpenedFramebuffer::new(
                front_buffer,
                back_buffer,
                dimensions,
                info.stride,
                info.bytes_per_pixel,
            ),
            borrowed: false,
        };

        framebuffer
    }

    pub fn borrow(&mut self, receiver: &mut Option<OpenedFramebuffer>) {
        if self.borrowed {
            receiver.take();
        } else {
            self.borrowed = true;
            let fb_ptr = &mut self.fb as *mut OpenedFramebuffer;

            // safety: this *does* create a aliased mutable reference, but it's safe because we keep track of the borrow and only allow one at a time
            *receiver = Some(unsafe { fb_ptr.read() });
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut OpenedFramebuffer> {
        if self.borrowed {
            None
        } else {
            Some(&mut self.fb)
        }
    }

    /// borrows the framebuffer mutably, even if it may lead to undefined behavior.
    ///
    ///  this is intended as a last resort in kernel panics, where we want to print the panic message to the framebuffer.
    pub unsafe fn now_or_never(&mut self) -> &mut OpenedFramebuffer {
        &mut self.fb
    }
}
