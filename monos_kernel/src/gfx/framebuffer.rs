use super::fonts::cozette;
use super::types::*;
use crate::mem::{self, VirtualAddress};

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

pub struct Framebuffer {
    text_color: Color,
    cursor: Position,

    dimensions: Dimension,
    info: FrameBufferInfo,
    buffer: &'static mut [u8],
}

impl Framebuffer {
    fn new(fb: RawFrameBuffer) -> Self {
        let info = fb.info();

        let buffer = fb.into_buffer();
        let buffer_virt = VirtualAddress::from_ptr(buffer);
        let buffer_phys = mem::translate_addr(buffer_virt).unwrap();

        let page = mem::Page::around(buffer_virt);
        let frame = mem::Frame::around(buffer_phys);

        use mem::PageTableFlags;
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::WRITE_THROUGH
            | PageTableFlags::CACHE_DISABLE;

        // unmap the existing mapping from the bootloader
        match mem::unmap(page) {
            Ok(_) => {}
            Err(mem::UnmapError::NotMapped) => {}
            Err(_) => panic!("failed to unmap frame buffer"),
        }
        unsafe { mem::map_to(&page, &frame, flags) }.expect("failed to map frame buffer");

        let ptr = page.start_address().as_mut_ptr::<u8>();
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, info.byte_len) };

        let mut framebuffer = Self {
            text_color: Color::new(255, 255, 255),
            cursor: Position::new(0, 0),

            dimensions: Dimension::new(info.width / 2, info.height / 2),
            info,
            buffer,
        };

        framebuffer.clear(&Color::new(0, 0, 0));

        framebuffer
    }

    #[inline]
    #[allow(dead_code)]
    pub fn dimensions(&self) -> &Dimension {
        &self.dimensions
    }

    pub fn clear(&mut self, color: &Color) {
        for y in 0..self.dimensions.height {
            for x in 0..self.dimensions.width {
                self.draw_pixel(&Position::new(x, y), &color);
            }
        }
    }

    pub fn set_color(&mut self, color: &Color) {
        self.text_color = color.clone();
    }

    pub fn write_string(&mut self, string: &str) {
        for character in string.chars() {
            if character == '\n' {
                self.cursor.x = 0;
                self.cursor.y += 13; // just assume the font is 13px tall for now
            } else {
                self.draw_char(&self.text_color.clone(), character);
            }
        }
    }

    pub fn draw_char(&mut self, color: &Color, character: char) {
        if let Some(char) = cozette::get_char(character) {
            let char = Character::from_raw(char);

            let mut bit_offset = 0;
            let mut byte_offset = 0;
            for y in 0..char.height {
                for x in 0..char.width {
                    let byte = char.data.get(byte_offset).unwrap_or(&1);
                    if byte & (1 << bit_offset) == 0 {
                        self.draw_pixel(
                            &Position {
                                x: self.cursor.x + x,
                                y: self.cursor.y + y,
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

            self.cursor.x += char.width;
        }
    }

    pub fn draw_pixel(&mut self, position: &Position, color: &Color) {
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
        let pixel_bytes = &mut self.buffer[byte_offset..];
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
        $crate::gfx::framebuffer().write_fmt(format_args!($($arg)*)).unwrap();
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
        $crate::gfx::framebuffer().set_color(&$crate::gfx::Color::new(230, 50, 50));
        $crate::print!($($arg)*);
        $crate::gfx::framebuffer().set_color(&$crate::gfx::Color::new(255, 255, 255));
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}
