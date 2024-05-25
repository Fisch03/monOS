use super::fonts::cozette;
use super::types::*;
use crate::mem::{self, VirtualAddress};

use alloc::collections::VecDeque;
use bootloader_api::info::{FrameBuffer as RawFrameBuffer, FrameBufferInfo, PixelFormat};
use core::slice;
use spin::{Mutex, MutexGuard, Once};

static FRAMEBUFFER: Once<Mutex<Framebuffer>> = Once::new();

pub fn init(fb: RawFrameBuffer) {
    FRAMEBUFFER.call_once(|| Mutex::new(Framebuffer::new(fb)));
}

pub fn framebuffer() -> MutexGuard<'static, Framebuffer> {
    FRAMEBUFFER
        .get()
        .expect("Framebuffer not initialized")
        .lock()
}

const CHAR_WIDTH: usize = 6;
const CHAR_HEIGHT: usize = 13;

// dimension is usually 640x360 (almost like god intended)
const TEXT_BUFFER_WIDTH: usize = 640 / CHAR_WIDTH;
const TEXT_BUFFER_HEIGHT: usize = 360 / CHAR_HEIGHT;

pub struct Framebuffer {
    dimensions: Dimension,
    info: FrameBufferInfo,

    front_buffer: &'static mut [u8],
    back_buffer: &'static mut [u8],

    text_buffer: VecDeque<char>,
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

        let back_buffer_virt = VirtualAddress::new(0x123456780000);
        let mut back_buffer_page = mem::Page::around(back_buffer_virt);

        let back_buffer = back_buffer_page.start_address().as_mut_ptr::<u8>();
        let back_buffer = unsafe { slice::from_raw_parts_mut(back_buffer, info.byte_len) };

        use mem::PageTableFlags;
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::WRITE_THROUGH
            | PageTableFlags::CACHE_DISABLE;

        loop {
            // (try) to unmap the existing mapping from the bootloader
            match mem::unmap(front_buffer_page) {
                Ok(_) => {}
                Err(mem::UnmapError::NotMapped) => {}
                Err(_) => panic!("failed to unmap frame buffer"),
            }
            unsafe { mem::map_to(&front_buffer_page, &front_buffer_frame, flags) }
                .expect("failed to map frame buffer");

            let back_buffer_frame =
                mem::alloc_frame().expect("failed to allocate back buffer frame");
            unsafe { mem::map_to(&back_buffer_page, &back_buffer_frame, flags) }
                .expect("failed to map back buffer");

            if front_buffer_page == front_buffer_end_page {
                break;
            }

            front_buffer_frame = front_buffer_frame.next();
            front_buffer_page = front_buffer_page.next();
            back_buffer_page = back_buffer_page.next();
        }
        let mut framebuffer = Self {
            dimensions: Dimension::new(info.width / 2, info.height / 2),
            info,

            front_buffer,
            back_buffer,

            text_buffer: VecDeque::with_capacity(TEXT_BUFFER_WIDTH * TEXT_BUFFER_HEIGHT),
        };

        framebuffer.update();

        framebuffer
    }

    fn clear(&mut self) {
        // for some reason, the builtin fill function is *really* slow, so we'll do it manually
        // self.back_buffer.fill(0);

        unsafe { core::ptr::write_bytes(self.back_buffer.as_mut_ptr(), 0, self.back_buffer.len()) };
    }

    fn swap_buffers(&mut self) {
        // this fares a lot better than fill, but its still slower than manually copying
        // self.front_buffer.copy_from_slice(self.back_buffer);

        unsafe {
            core::ptr::copy_nonoverlapping(
                self.back_buffer.as_ptr(),
                self.front_buffer.as_mut_ptr(),
                self.back_buffer.len(),
            );
        }
    }

    pub fn update(&mut self) {
        self.clear();

        let mut position = Position::new(0, 0);
        for i in 0..self.text_buffer.len() {
            let character = self.text_buffer.get(i);
            match character {
                Some('\n') => {
                    position.x = 0;
                    position.y += 1;
                }
                Some(character) => {
                    self.draw_char(&Color::new(255, 255, 255), *character, &position, false);
                    position.x += 1;
                    if position.x >= TEXT_BUFFER_WIDTH {
                        position.x = 0;
                        position.y += 1;
                    }
                }
                None => break,
            }
        }

        self.swap_buffers();
    }

    pub fn write_string(&mut self, s: &str) {
        for character in s.chars() {
            self.text_buffer.push_back(character);
            if self.text_buffer.len() > TEXT_BUFFER_WIDTH * TEXT_BUFFER_HEIGHT {
                self.text_buffer.pop_front();
            }
        }
    }

    fn draw_char(&mut self, color: &Color, character: char, position: &Position, overdraw: bool) {
        if let Some(char) = cozette::get_char(character) {
            let char = Character::from_raw(char);

            let base_x = position.x * CHAR_WIDTH;
            let base_y = position.y * CHAR_HEIGHT;

            if overdraw {
                for y in 0..CHAR_HEIGHT {
                    for x in 0..CHAR_WIDTH {
                        self.draw_pixel(
                            &Position {
                                x: base_x + x,
                                y: base_y + y,
                            },
                            &Color::new(0, 0, 0),
                        );
                    }
                }
            }

            let mut bit_offset = 0;
            let mut byte_offset = 0;
            for y in 0..char.height {
                for x in 0..char.width {
                    let byte = char.data.get(byte_offset).unwrap_or(&1);
                    if byte & (1 << bit_offset) == 0 {
                        self.draw_pixel(
                            &Position {
                                x: base_x + x,
                                y: base_y + y,
                            },
                            color,
                        );
                    }
                    bit_offset += 1;
                    if bit_offset % 8 == 0 {
                        byte_offset += 1;
                        bit_offset = 0;
                    }
                }
            }
        }
    }

    fn draw_pixel(&mut self, position: &Position, color: &Color) {
        if position.x >= self.dimensions.width || position.y >= self.dimensions.height {
            return;
        }

        let position = Position::new(position.x * 2, position.y * 2);

        let y_offset_lower = position.y * self.info.stride;
        let y_offset_upper = y_offset_lower + self.info.stride;

        let pixel_offsets = [
            y_offset_lower + position.x,
            y_offset_lower + position.x + 1,
            y_offset_upper + position.x,
            y_offset_upper + position.x + 1,
        ];

        pixel_offsets.iter().for_each(|offset| {
            self.draw_pixel_raw(offset * self.info.bytes_per_pixel, color);
        });
    }

    fn draw_pixel_raw(&mut self, byte_offset: usize, color: &Color) {
        let pixel_bytes = &mut self.back_buffer[byte_offset..];
        match self.info.pixel_format {
            PixelFormat::Rgb => {
                pixel_bytes[0] = color.r;
                pixel_bytes[1] = color.g;
                pixel_bytes[2] = color.b;
            }
            PixelFormat::Bgr => {
                pixel_bytes[0] = color.b;
                pixel_bytes[1] = color.g;
                pixel_bytes[2] = color.r;
            }
            PixelFormat::U8 => {
                pixel_bytes[0] = color.r / 3 + color.g / 3 + color.b / 3;
            }
            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => {
                pixel_bytes[red_position as usize] = color.r;
                pixel_bytes[green_position as usize] = color.g;
                pixel_bytes[blue_position as usize] = color.b;
            }
            _ => {
                panic!("Unsupported pixel format");
            }
        }
    }
}

use core::fmt;
impl fmt::Write for Framebuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::interrupts::without_interrupts;

        without_interrupts(|| {
            $crate::gfx::framebuffer().write_fmt(format_args!($($arg)*)).unwrap();
        });
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        $crate::print!($($arg)*);
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}
