use crate::{types::*, Image, ImageFormat};
use monos_std::messaging::*;

#[derive(Debug)]
pub enum FramebufferRequest<'a> {
    Open(&'a mut Option<Framebuffer<'static>>),
    SubmitFrame(&'a Framebuffer<'a>),
}

#[derive(Debug)]
pub enum FramebufferResponse {
    OK,
}

impl MessageData for FramebufferRequest<'_> {
    fn into_message(self) -> (u64, u64, u64, u64) {
        match self {
            FramebufferRequest::Open(fb) => {
                let fb_ptr = fb as *mut _ as u64;
                (0, 0, 0, fb_ptr)
            }
            FramebufferRequest::SubmitFrame(fb) => {
                let fb_ptr = fb as *const _ as u64;
                (1, 0, 0, fb_ptr)
            }
        }
    }

    unsafe fn from_message(message: &Message) -> Option<Self> {
        match message.data {
            (0, 0, 0, fb_ptr) => {
                let fb = &mut *(fb_ptr as *mut Option<Framebuffer>);
                Some(FramebufferRequest::Open(fb))
            }
            (1, 0, 0, fb_ptr) => {
                let fb = &*(fb_ptr as *const Framebuffer);
                Some(FramebufferRequest::SubmitFrame(fb))
            }
            _ => None,
        }
    }
}

impl MessageData for FramebufferResponse {
    fn into_message(self) -> (u64, u64, u64, u64) {
        match self {
            FramebufferResponse::OK => (0, 0, 0, 0),
        }
    }

    unsafe fn from_message(_message: &Message) -> Option<Self> {
        match _message.data {
            (0, 0, 0, 0) => Some(FramebufferResponse::OK),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FramebufferFormat {
    pub bytes_per_pixel: u64,
    pub stride: u64,

    pub r_position: usize,
    pub g_position: usize,
    pub b_position: usize,
    pub a_position: Option<usize>,
}

pub struct Framebuffer<'a> {
    buffer: &'a mut [u8],

    dimensions: Dimension,

    format: FramebufferFormat,
}

impl core::fmt::Debug for Framebuffer<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Framebuffer")
            .field("dimensions", &self.dimensions)
            .field("format", &self.format)
            .finish()
    }
}

impl<'a> Framebuffer<'a> {
    pub fn new(buffer: &'a mut [u8], dimensions: Dimension, format: FramebufferFormat) -> Self {
        Self {
            buffer,
            dimensions,
            format,
        }
    }

    #[inline(always)]
    pub fn format(&self) -> &FramebufferFormat {
        &self.format
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
    pub fn byte_len(&self) -> usize {
        self.buffer.len()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        // for some reason, the builtin fill function is *really* slow, so we'll do it manually
        // self.back_buffer.fill(0);

        unsafe { core::ptr::write_bytes(self.buffer.as_mut_ptr(), 0, self.buffer.len()) };
    }

    #[inline(always)]
    pub fn clear_alpha(&mut self) {
        if let Some(a_position) = self.format.a_position {
            for i in (0..self.buffer.len()).step_by(self.format.bytes_per_pixel as usize) {
                self.buffer[i + a_position] = 255;
            }
        }
    }

    #[inline(always)]
    pub fn clear_with(&mut self, fb: &Framebuffer) {
        assert!(self.buffer.len() == fb.buffer.len());
        assert!(self.format == fb.format);

        unsafe {
            core::ptr::copy_nonoverlapping(
                fb.buffer.as_ptr(),
                self.buffer.as_mut_ptr(),
                self.buffer.len(),
            )
        };
    }

    #[inline(always)]
    pub fn clear_region(&mut self, rect: &Rect, fb: &Framebuffer) {
        assert!(self.format == fb.format);

        let mut line_start = ((rect.min.y * self.format.stride as i64 + rect.min.x)
            * self.format.bytes_per_pixel as i64) as usize;
        let mut line_pos = line_start;
        let len = rect.dimensions().width as usize * self.format.bytes_per_pixel as usize;

        for _y in rect.min.y..rect.max.y {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    fb.buffer[line_pos..line_pos + len].as_ptr(),
                    self.buffer[line_pos..line_pos + len].as_mut_ptr(),
                    len,
                )
            };

            line_start += (self.format.stride * self.format.bytes_per_pixel) as usize;
            if line_start >= self.buffer.len() {
                return;
            }
            line_pos = line_start;
        }
    }

    pub fn draw_rect(&mut self, rect: &Rect, color: &Color) {
        let mut rect = rect.clone();
        rect.min.x = rect.min.x.max(0).min(self.dimensions.width as i64);
        rect.min.y = rect.min.y.max(0).min(self.dimensions.height as i64);
        rect.max.x = rect.max.x.max(0).min(self.dimensions.width as i64);
        rect.max.y = rect.max.y.max(0).min(self.dimensions.height as i64);

        // TODO: optimize this
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                self.draw_pixel(&Position { x, y }, color);
            }
        }
    }

    pub fn draw_box(&mut self, rect: &Rect, color: &Color) {
        // TODO: optimize this
        for x in rect.min.x..rect.max.x {
            self.draw_pixel(&Position { x, y: rect.min.y }, color);
            self.draw_pixel(
                &Position {
                    x,
                    y: rect.max.y - 1,
                },
                color,
            );
        }
        for y in rect.min.y..rect.max.y {
            self.draw_pixel(&Position { x: rect.min.x, y }, color);
            self.draw_pixel(
                &Position {
                    x: rect.max.x - 1,
                    y,
                },
                color,
            );
        }
    }

    pub fn draw_img(&mut self, image: &Image, position: &Position) {
        let (mut image_data, alpha_val) = match &image.data {
            ImageFormat::RGB { data, alpha_val } => (data.iter(), alpha_val),
            _ => return,
        };

        let skip_y = (-position.y).max(0);
        let skip_x = (-position.x).max(0);

        if skip_y as u32 >= image.dimensions().height
            || skip_x as u32 >= image.dimensions().width
            || position.y >= self.dimensions.height as i64
            || position.x >= self.dimensions.width as i64
        {
            return;
        }

        let mut line_start = (((position.y + skip_y) * self.format.stride as i64
            + (position.x + skip_x))
            * self.format.bytes_per_pixel as i64) as usize;
        let mut line_pos = line_start;
        let max_x = (self.dimensions.width - position.x as u32).min(image.dimensions().width);

        let mut skip_y_current = skip_y;

        for _y in 0..image.dimensions().height {
            let mut skip_y = false;
            if skip_y_current > 0 {
                skip_y_current -= 1;
                skip_y = true
            }

            let mut skip_x_current = skip_x;
            for x in 0..image.dimensions().width {
                let r = *image_data.next().unwrap_or(&0);
                let g = *image_data.next().unwrap_or(&0);
                let b = *image_data.next().unwrap_or(&0);

                if skip_x_current > 0 {
                    skip_x_current -= 1;
                } else if x < max_x {
                    let mut skip_alpha = false;
                    if let Some(alpha_val) = alpha_val {
                        if r == alpha_val.r && g == alpha_val.g && b == alpha_val.b {
                            skip_alpha = true;
                        }
                    }
                    if !skip_alpha {
                        let pixel_bytes = &mut self.buffer[line_pos..];
                        pixel_bytes[self.format.r_position] = r;
                        pixel_bytes[self.format.g_position] = g;
                        pixel_bytes[self.format.b_position] = b;
                    }

                    line_pos += self.format.bytes_per_pixel as usize;
                }
            }

            if !skip_y {
                line_start += (self.format.stride * self.format.bytes_per_pixel) as usize;
                if line_start >= self.buffer.len() {
                    return;
                }
                line_pos = line_start;
            }
        }
    }

    pub fn draw_char<Font: crate::fonts::Font>(
        &mut self,
        color: &Color,
        character: char,
        position: &Position,
    ) {
        if let Some(char) = Font::get_char(character) {
            let char = crate::fonts::Character::from_raw(char);
            let mut bit_offset = 0;
            let mut byte_offset = 0;

            for y in 0..char.height {
                for x in 0..char.width {
                    let byte = char.data.get(byte_offset).unwrap_or(&1);
                    if byte & (1 << bit_offset) == 0 {
                        self.draw_pixel(
                            &Position {
                                x: position.x + x as i64,
                                y: position.y + y as i64,
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

    pub fn draw_str<Font: crate::fonts::Font>(
        &mut self,
        color: &Color,
        string: &str,
        position: &Position,
    ) {
        use crate::fonts::Character;

        let chars = string
            .chars()
            .filter_map(|c| Font::get_char(c).map(|c| Character::from_raw(c)))
            .collect::<Vec<_>>();

        let mut line_start = ((position.y * self.format.stride as i64 + position.x)
            * self.format.bytes_per_pixel as i64) as usize;
        let mut line_pos = line_start;

        for y in 0..Font::CHAR_HEIGHT as usize {
            for c in &chars {
                let next_line_pos =
                    line_pos + Font::CHAR_WIDTH as usize * self.format.bytes_per_pixel as usize;
                for x in 0..c.width {
                    let c_bit_offset = (y * c.width + x) as usize;
                    let c_byte_offset = c_bit_offset / 8;
                    let c_bit_offset = c_bit_offset % 8;

                    let byte = c.data.get(c_byte_offset).unwrap_or(&1);

                    if byte & (1 << c_bit_offset) == 0 {
                        let pixel_bytes = &mut self.buffer[line_pos..];
                        pixel_bytes[self.format.r_position] = color.r;
                        pixel_bytes[self.format.g_position] = color.g;
                        pixel_bytes[self.format.b_position] = color.b;
                    }
                    line_pos += self.format.bytes_per_pixel as usize;
                }
                line_pos = next_line_pos;
            }

            line_start += (self.format.stride * self.format.bytes_per_pixel) as usize;
            if line_start >= self.buffer.len() {
                return;
            }
            line_pos = line_start;
        }
    }

    pub fn draw_pixel(&mut self, position: &Position, color: &Color) {
        if position.x >= self.dimensions.width.into() || position.y >= self.dimensions.height.into()
        {
            return;
        }

        let y = position.y as u64 * self.format.stride;

        let offset = y + position.x as u64;

        self.draw_pixel_raw((offset * self.format.bytes_per_pixel) as usize, color);
    }

    #[inline(always)]
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
