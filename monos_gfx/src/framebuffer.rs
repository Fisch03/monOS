use alloc::collections::VecDeque;

use crate::fonts;
use crate::types::*;

const CHAR_WIDTH: usize = 6;
const CHAR_HEIGHT: usize = 13;

pub struct OpenedFramebuffer {
    dimensions: Dimension,

    stride: usize,
    bytes_per_pixel: usize,

    front_buffer: &'static mut [u8],
    back_buffer: &'static mut [u8],

    text_buffer: VecDeque<char>,
}

impl OpenedFramebuffer {
    pub fn new(
        front_buffer: &'static mut [u8],
        back_buffer: &'static mut [u8],
        dimensions: Dimension,
        stride: usize,
        bytes_per_pixel: usize,
    ) -> Self {
        let text_buffer_width = dimensions.width / CHAR_WIDTH;
        let text_buffer_height = dimensions.height / CHAR_HEIGHT;

        Self {
            dimensions,
            stride,
            bytes_per_pixel,

            front_buffer,
            back_buffer,

            text_buffer: VecDeque::with_capacity(text_buffer_width * text_buffer_height),
        }
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
                    if position.x >= self.dimensions.width / CHAR_WIDTH {
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
            if self.text_buffer.len()
                > self.dimensions.width / CHAR_WIDTH * self.dimensions.height / CHAR_HEIGHT
            {
                self.text_buffer.pop_front();
            }
        }
    }

    fn draw_char(&mut self, color: &Color, character: char, position: &Position, overdraw: bool) {
        if let Some(char) = fonts::cozette::get_char(character) {
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

        let y_offset_lower = position.y * self.stride;
        let y_offset_upper = y_offset_lower + self.stride;

        let pixel_offsets = [
            y_offset_lower + position.x,
            y_offset_lower + position.x + 1,
            y_offset_upper + position.x,
            y_offset_upper + position.x + 1,
        ];

        pixel_offsets.iter().for_each(|offset| {
            self.draw_pixel_raw(offset * self.bytes_per_pixel, color);
        });
    }

    fn draw_pixel_raw(&mut self, byte_offset: usize, color: &Color) {
        let pixel_bytes = &mut self.back_buffer[byte_offset..];
        // match self.info.pixel_format {
        //     PixelFormat::Rgb => {
        pixel_bytes[0] = color.r;
        pixel_bytes[1] = color.g;
        pixel_bytes[2] = color.b;
        //     }
        //     PixelFormat::Bgr => {
        //         pixel_bytes[0] = color.b;
        //         pixel_bytes[1] = color.g;
        //         pixel_bytes[2] = color.r;
        //     }
        //     PixelFormat::U8 => {
        //         pixel_bytes[0] = color.r / 3 + color.g / 3 + color.b / 3;
        //     }
        //     PixelFormat::Unknown {
        //         red_position,
        //         green_position,
        //         blue_position,
        //     } => {
        //         pixel_bytes[red_position as usize] = color.r;
        //         pixel_bytes[green_position as usize] = color.g;
        //         pixel_bytes[blue_position as usize] = color.b;
        //     }
        //     _ => {
        //         panic!("Unsupported pixel format");
        //     }
        // }
    }
}

use core::fmt;
impl fmt::Write for OpenedFramebuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
