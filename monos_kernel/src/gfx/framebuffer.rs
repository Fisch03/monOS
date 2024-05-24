use super::fonts::cozette;
use super::types::*;
use crate::mem::{self, VirtualAddress};

use alloc::vec::Vec;
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
    text_color: Color,
    cursor: Position,

    dimensions: Dimension,
    info: FrameBufferInfo,

    input_buffer: Vec<char>,
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

            input_buffer: Vec::new(),
            buffer,
        };

        framebuffer.clear(&Color::new(0, 0, 0));

        framebuffer.draw_char(
            &Color::new(108, 207, 240),
            '>',
            &Position::new(1, TEXT_BUFFER_HEIGHT - 1),
            false,
        );
        framebuffer.draw_char(
            &Color::new(108, 207, 240),
            '_',
            &Position::new(2, TEXT_BUFFER_HEIGHT - 1),
            false,
        );

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

    pub fn add_input_char(&mut self, character: char) {
        self.input_buffer.push(character);
        self.draw_char(
            &Color::new(108, 207, 240),
            character,
            &Position::new(self.input_buffer.len() + 1, TEXT_BUFFER_HEIGHT - 1),
            true,
        );
        self.draw_char(
            &Color::new(108, 207, 240),
            '_',
            &Position::new(self.input_buffer.len() + 2, TEXT_BUFFER_HEIGHT - 1),
            false,
        );
    }

    pub fn delete_input_char(&mut self) {
        if let Some(_) = self.input_buffer.pop() {
            self.draw_char(
                &Color::new(108, 207, 240),
                ' ',
                &Position::new(self.input_buffer.len() + 3, TEXT_BUFFER_HEIGHT - 1),
                true,
            );

            self.draw_char(
                &Color::new(108, 207, 240),
                '_',
                &Position::new(self.input_buffer.len() + 2, TEXT_BUFFER_HEIGHT - 1),
                true,
            );
        }
    }

    pub fn confirm_input(&mut self) {
        self.draw_char(
            &Color::new(108, 207, 240),
            ' ',
            &Position::new(self.input_buffer.len() + 2, TEXT_BUFFER_HEIGHT - 1),
            true,
        );

        let cmd = self.input_buffer.iter().collect::<alloc::string::String>();
        crate::dbg!(cmd);
        self.input_buffer.clear();

        self.scroll_with_input();

        self.draw_char(
            &Color::new(108, 207, 240),
            '>',
            &Position::new(1, TEXT_BUFFER_HEIGHT - 1),
            false,
        );
        self.draw_char(
            &Color::new(108, 207, 240),
            '_',
            &Position::new(2, TEXT_BUFFER_HEIGHT - 1),
            false,
        );
    }

    pub fn set_color(&mut self, color: &Color) {
        self.text_color = color.clone();
    }

    pub fn write_string(&mut self, string: &str) {
        for character in string.chars() {
            if character == '\n' {
                self.new_line();
            } else {
                self.draw_char(
                    &self.text_color.clone(),
                    character,
                    &Position::new(self.cursor.x, self.cursor.y),
                    false,
                );
                self.cursor.x += 1;
                if self.cursor.x >= TEXT_BUFFER_WIDTH {
                    self.new_line();
                }
            }
        }
    }

    pub fn draw_char(
        &mut self,
        color: &Color,
        character: char,
        position: &Position,
        overdraw: bool,
    ) {
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

    pub fn new_line(&mut self) {
        self.cursor.x = 0;
        self.cursor.y += 1;
        if self.cursor.y >= TEXT_BUFFER_HEIGHT - 1 {
            self.scroll();
        }
    }

    pub fn scroll(&mut self) {
        for x in 0..self.dimensions.width {
            for y in 0..self.dimensions.height - CHAR_HEIGHT * 2 {
                let pos = Position::new(x, y + CHAR_HEIGHT);
                self.draw_pixel(&Position::new(x, y), &self.get_pixel(&pos));
            }
        }

        self.cursor.y -= 1;
    }

    pub fn scroll_with_input(&mut self) {
        for x in 0..self.dimensions.width {
            for y in 0..self.dimensions.height - CHAR_HEIGHT * 1 {
                let pos = Position::new(x, y + CHAR_HEIGHT);
                self.draw_pixel(&Position::new(x, y), &self.get_pixel(&pos));
            }
        }

        self.cursor.y -= 1;
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

    pub fn get_pixel(&self, position: &Position) -> Color {
        let position = Position::new(position.x * 2, position.y * 2);
        let y_offset = position.y * self.info.stride;
        let pixel_offset = (y_offset + position.x) * self.info.bytes_per_pixel;
        let pixel_bytes = &self.buffer[pixel_offset..];

        match self.info.pixel_format {
            PixelFormat::Rgb => Color::new(pixel_bytes[0], pixel_bytes[1], pixel_bytes[2]),
            PixelFormat::Bgr => Color::new(pixel_bytes[2], pixel_bytes[1], pixel_bytes[0]),
            PixelFormat::U8 => Color::new(pixel_bytes[0], pixel_bytes[0], pixel_bytes[0]),
            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => Color::new(
                pixel_bytes[red_position as usize],
                pixel_bytes[green_position as usize],
                pixel_bytes[blue_position as usize],
            ),
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
