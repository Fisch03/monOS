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
}

use core::fmt;
impl fmt::Debug for OpenedFramebuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenedFramebuffer")
            .field("dimensions", &self.dimensions)
            .field("stride", &self.stride)
            .field("bytes_per_pixel", &self.bytes_per_pixel)
            .field("front_buffer length", &self.front_buffer.len())
            .field("back_buffer length", &self.back_buffer.len())
            .finish()
    }
}

impl OpenedFramebuffer {
    pub fn new(
        front_buffer: &'static mut [u8],
        back_buffer: &'static mut [u8],
        dimensions: Dimension,
        stride: usize,
        bytes_per_pixel: usize,
    ) -> Self {
        back_buffer.fill(0);

        let mut fb = Self {
            dimensions,
            stride,
            bytes_per_pixel,

            front_buffer,
            back_buffer,
        };

        fb.update();

        fb
    }

    #[inline]
    fn clear(&mut self) {
        // for some reason, the builtin fill function is *really* slow, so we'll do it manually
        // self.back_buffer.fill(0);

        unsafe { core::ptr::write_bytes(self.back_buffer.as_mut_ptr(), 0, self.back_buffer.len()) };
    }

    #[inline]
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

    #[inline]
    pub fn update(&mut self) {
        self.swap_buffers();
        self.clear();
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

    #[inline]
    pub fn draw_pixel(&mut self, position: &Position, color: &Color) {
        if position.x >= self.dimensions.width || position.y >= self.dimensions.height {
            return;
        }

        let y_offset_lower = position.y * self.stride;
        let offset = y_offset_lower + position.x;

        self.draw_pixel_raw(offset * self.bytes_per_pixel, color);
    }

    #[inline]
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
