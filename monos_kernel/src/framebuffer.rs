use crate::mem::{self, PageTableFlags, VirtualAddress};

use bootloader_api::info::FrameBuffer as RawFrameBuffer;
use core::slice;
use spin::{Mutex, MutexGuard, Once};

use monos_gfx::{types::*, Framebuffer};

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

    back_buffer_start_frame: mem::Frame,

    borrowed: Option<u32>,
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

        let back_buffer_virt = mem::alloc_vmem(info.byte_len as u64);
        let mut back_buffer_page = mem::Page::around(back_buffer_virt);
        let back_buffer_start_frame =
            mem::alloc_frames(info.byte_len as usize / 4096).expect("no memory for back buffer");
        let mut back_buffer_frame = back_buffer_start_frame;

        let back_buffer =
            unsafe { slice::from_raw_parts_mut(back_buffer_virt.as_mut_ptr(), info.byte_len) };

        use mem::PageTableFlags;
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::WRITE_THROUGH
            | PageTableFlags::CACHE_DISABLE;

        loop {
            unsafe { mem::map_to(&front_buffer_page, &front_buffer_frame, flags) }
                .expect("failed to map front buffer");

            unsafe { mem::map_to(&back_buffer_page, &back_buffer_frame, flags) }
                .expect("failed to map kernel back buffer");

            if front_buffer_page == front_buffer_end_page {
                break;
            }

            front_buffer_frame = front_buffer_frame.next();
            front_buffer_page = front_buffer_page.next();
            back_buffer_frame = back_buffer_frame.next();
            back_buffer_page = back_buffer_page.next();
        }

        let framebuffer = Self {
            front_buffer,
            framebuffer: Framebuffer::new(
                back_buffer,
                Dimension::new(info.width, info.height),
                info.stride as usize,
                info.bytes_per_pixel as usize,
            ),
            back_buffer_start_frame,
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

    pub fn borrow(
        &mut self,
        process_id: u32,
        receiver: &mut Option<Framebuffer>,
        mapper: &mut mem::Mapper,
        start: VirtualAddress,
    ) {
        if self.borrowed.is_some() {
            receiver.take();
        } else {
            use mem::MapTo;

            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::WRITE_THROUGH
                | PageTableFlags::CACHE_DISABLE
                | PageTableFlags::USER_ACCESSIBLE;

            self.borrowed = Some(process_id);

            let mut frame = self.back_buffer_start_frame;
            let mut page = mem::Page::around(start);
            let end_page =
                mem::Page::around(page.start_address() + self.framebuffer.buffer().len() as u64)
                    .next();

            loop {
                unsafe { mapper.map_to(&page, &frame, flags) }.expect("failed to map framebuffer");

                if page == end_page {
                    break;
                }

                frame = frame.next();
                page = page.next();
            }

            let framebuffer = Framebuffer::new(
                unsafe {
                    slice::from_raw_parts_mut(start.as_mut_ptr(), self.framebuffer.buffer().len())
                },
                self.framebuffer.dimensions(),
                self.framebuffer.stride(),
                self.framebuffer.bytes_per_pixel(),
            );

            *receiver = Some(framebuffer);
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
