use crate::mem::{self, PageTableFlags, VirtualAddress};

use crate::process::messaging::{add_system_port, GenericMessage, PartialSendChannelHandle};
use bootloader_api::info::FrameBuffer as RawFrameBuffer;
use bootloader_api::info::PixelFormat;
use core::slice;
use spin::{Mutex, MutexGuard, Once};

use monos_gfx::{types::*, Framebuffer, FramebufferFormat};

static FRAMEBUFFER: Once<Mutex<KernelFramebuffer>> = Once::new();

pub fn init(fb: RawFrameBuffer) {
    FRAMEBUFFER.call_once(|| Mutex::new(KernelFramebuffer::new(fb)));
}

pub fn get() -> Option<MutexGuard<'static, KernelFramebuffer<'static>>> {
    FRAMEBUFFER.get().map(|fb| fb.lock())
}

pub fn receive_message(message: GenericMessage) {
    let mut current_proc_guard = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc_guard.as_mut().unwrap();

    assert!(current_proc.id() == message.sender.target_process); //sanity check in case i ever change the way messaging works

    if let Some(mut fb_guard) = crate::framebuffer::get() {
        if fb_guard.borrowed.is_none() || fb_guard.borrowed.unwrap() != message.sender {
            crate::println!(
                "process {} tried to access framebuffer without borrowing it",
                message.sender.target_process
            );
            return;
        }

        use crate::process::messaging::{send, MessageData};
        use monos_gfx::framebuffer::{FramebufferRequest, FramebufferResponse};

        let requester = message.sender;
        let request = unsafe { FramebufferRequest::from_message(message).unwrap() };
        match request {
            FramebufferRequest::SubmitFrame(frame) => {
                fb_guard.submit_frame(frame.buffer());
            }
            FramebufferRequest::Open(fb) => {
                fb_guard.borrow(
                    requester,
                    fb,
                    current_proc.mapper(),
                    VirtualAddress::new(0x410000000000),
                );

                let return_message = GenericMessage {
                    sender: fb_guard.own_handle,
                    data: FramebufferResponse::OK.into_message(),
                };

                drop(current_proc_guard); // drop the guard before sending the message, to avoid deadlock
                send(return_message, requester);
            }
        }
    }
}

pub struct KernelFramebuffer<'a> {
    front_buffer: &'static mut [u8],
    framebuffer: Framebuffer<'a>,

    back_buffer_start_frame: mem::Frame,

    own_handle: PartialSendChannelHandle,
    borrowed: Option<PartialSendChannelHandle>,
}

impl<'a> KernelFramebuffer<'a> {
    fn new(fb: RawFrameBuffer) -> Self {
        let info = fb.info();

        let front_buffer = fb.into_buffer();
        let front_buffer_virt = VirtualAddress::from_ptr(front_buffer);
        let front_buffer_phys = mem::translate_addr(front_buffer_virt).unwrap();

        let mut front_buffer_page = mem::Page::around(front_buffer_virt);
        let mut front_buffer_frame = mem::Frame::around(front_buffer_phys);
        let front_buffer_end_page =
            mem::Page::around(front_buffer_virt + info.byte_len as u64).next();

        let back_buffer_virt = crate::FB_START;
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

        let own_handle = add_system_port(
            "sys.framebuffer",
            |borrower| {
                let mut fb_guard = get().unwrap();
                if fb_guard.borrowed.is_none() {
                    crate::println!("process {} borrowed framebuffer", borrower.target_process);
                    fb_guard.borrowed = Some(borrower);
                }
                fb_guard.own_handle
            },
            Some(receive_message),
        );

        let (r_position, g_position, b_position) = match info.pixel_format {
            PixelFormat::U8 => (0, 0, 0),
            PixelFormat::Rgb => (0, 1, 2),
            PixelFormat::Bgr => (2, 1, 0),
            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => (
                red_position as usize,
                green_position as usize,
                blue_position as usize,
            ),
            _ => panic!("unsupported pixel format"),
        };

        let framebuffer = Self {
            front_buffer,
            framebuffer: Framebuffer::new(
                back_buffer,
                Dimension::new(info.width as u32, info.height as u32),
                FramebufferFormat {
                    bytes_per_pixel: info.bytes_per_pixel as u64,
                    stride: info.stride as u64,

                    r_position,
                    g_position,
                    b_position,
                    a_position: None,
                },
            ),
            back_buffer_start_frame,
            own_handle,
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
        borrower: PartialSendChannelHandle,
        receiver: &mut Option<Framebuffer>,
        mapper: &mut mem::Mapper,
        start: VirtualAddress,
    ) {
        use mem::MapTo;

        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::WRITE_THROUGH
            | PageTableFlags::CACHE_DISABLE
            | PageTableFlags::USER_ACCESSIBLE;

        self.borrowed = Some(borrower);

        let mut frame = self.back_buffer_start_frame;
        let mut page = mem::Page::around(start);
        let end_page =
            mem::Page::around(page.start_address() + self.framebuffer.buffer().len() as u64).next();

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
            self.framebuffer.format().clone(),
        );

        *receiver = Some(framebuffer);
    }

    pub fn as_mut(&mut self) -> Option<&mut Framebuffer<'a>> {
        if self.borrowed.is_some() {
            None
        } else {
            Some(&mut self.framebuffer)
        }
    }

    /// borrows the framebuffer mutably, even if it may lead to undefined behavior.
    ///
    ///  this is intended as a last resort in kernel panics, where we want to print the panic message to the framebuffer.
    pub unsafe fn now_or_never(&mut self) -> &mut Framebuffer<'a> {
        &mut self.framebuffer
    }
}
