use crate::fonts;
use crate::types::*;
use alloc::{boxed::Box, vec, vec::Vec};

const CHAR_WIDTH: usize = 6;
const CHAR_HEIGHT: usize = 13;

#[derive(Debug)]
pub struct Framebuffer {
    buffer: &'static mut [u8],
    dimensions: Dimension,

    stride: usize,
    bytes_per_pixel: usize,
}

impl Framebuffer {
    pub fn new(
        buffer: &'static mut [u8],
        dimensions: Dimension,
        stride: usize,
        bytes_per_pixel: usize,
    ) -> Self {
        Self {
            buffer,
            dimensions,
            stride,
            bytes_per_pixel,
        }
    }

    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        self.buffer.as_ref()
    }

    #[inline(always)]
    pub fn dimensions(&self) -> Dimension {
        self.dimensions
    }

    #[inline(always)]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[inline(always)]
    pub fn bytes_per_pixel(&self) -> usize {
        self.bytes_per_pixel
    }

    #[inline(always)]
    pub fn byte_len(&self) -> usize {
        self.buffer.len()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        // for some reason, the builtin fill function is *really* slow, so we'll do it manually
        // self.back_buffer.fill(0);

        unsafe { core::ptr::write_bytes(self.buffer.as_mut_ptr(), 0, self.buffer.len()) };
    }

    fn draw_char(&mut self, color: &Color, character: char, position: &Position, overdraw: bool) {
        let position_x = position.x as usize;
        let position_y = position.y as usize;

        if let Some(char) = fonts::cozette::get_char(character) {
            let char = Character::from_raw(char);

            let base_x = position_x * CHAR_WIDTH;
            let base_y = position_y * CHAR_HEIGHT;

            if overdraw {
                for y in 0..CHAR_HEIGHT {
                    for x in 0..CHAR_WIDTH {
                        self.draw_pixel(
                            &Position {
                                x: (base_x + x) as i64,
                                y: (base_y + y) as i64,
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
                                x: (base_x + x) as i64,
                                y: (base_y + y) as i64,
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

    #[inline]
    pub fn draw_pixel(&mut self, position: &Position, color: &Color) {
        let position_x = position.x as usize;
        let position_y = position.y as usize;

        if position_x >= self.dimensions.width || position_y >= self.dimensions.height {
            return;
        }

        let y_offset_lower = position_y * self.stride;
        let offset = y_offset_lower + position_x;

        self.draw_pixel_raw(offset * self.bytes_per_pixel, color);
    }

    #[inline]
    fn draw_pixel_raw(&mut self, byte_offset: usize, color: &Color) {
        let pixel_bytes = &mut self.buffer[byte_offset..];
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
