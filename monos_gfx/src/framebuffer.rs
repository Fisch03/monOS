use crate::{
    text::{Character, Font},
    types::*,
    Image, ImageFormat,
};
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

    pub fn clear(&mut self) {
        // for some reason, the builtin fill function is *really* slow, so we'll do it manually
        // self.back_buffer.fill(0);

        unsafe { core::ptr::write_bytes(self.buffer.as_mut_ptr(), 0, self.buffer.len()) };
    }

    #[inline(always)]
    pub fn get_pixel(&self, position: Position) -> Color {
        let byte_offset = self.pos_to_offset(position) as usize;

        let pixel_bytes = &self.buffer[byte_offset..];
        Color::new(
            pixel_bytes[self.format.r_position],
            pixel_bytes[self.format.g_position],
            pixel_bytes[self.format.b_position],
        )
    }

    #[inline(always)]
    pub fn get_pixel_alpha(&self, position: Position) -> (Color, u8) {
        let byte_offset = self.pos_to_offset(position) as usize;

        let pixel_bytes = &self.buffer[byte_offset..];
        (
            Color::new(
                pixel_bytes[self.format.r_position],
                pixel_bytes[self.format.g_position],
                pixel_bytes[self.format.b_position],
            ),
            if let Some(a_position) = self.format.a_position {
                pixel_bytes[a_position]
            } else {
                255
            },
        )
    }

    #[inline(always)]
    fn pos_to_offset(&self, pos: Position) -> i64 {
        (pos.y * self.format.stride as i64 + pos.x) * self.format.bytes_per_pixel as i64
    }

    pub fn clear_alpha(&mut self) {
        if let Some(a_position) = self.format.a_position {
            for i in (0..self.buffer.len()).step_by(self.format.bytes_per_pixel as usize) {
                self.buffer[i + a_position] = 255;
            }
        }
    }

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

    pub fn clear_region(&mut self, rect: &Rect, fb: &Framebuffer) {
        assert!(self.buffer.len() == fb.buffer.len());
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

    pub fn draw_fb(&mut self, fb: &Framebuffer, position: Position) {
        self.draw_fb_clipped(fb, position, Rect::from_dimensions(self.dimensions))
    }

    pub fn draw_fb_clipped(&mut self, fb: &Framebuffer, position: Position, clip: Rect) {
        let clip = Rect {
            min: Position::new(clip.min.x.max(0), clip.min.y.max(0)),
            max: Position::new(
                clip.max.x.min(self.dimensions.width as i64),
                clip.max.y.min(self.dimensions.height as i64),
            ),
        };

        let mut current_position = position.clone();

        for y in 0..fb.dimensions.height as i64 {
            if current_position.y >= clip.max.y as i64 {
                return;
            }
            for x in 0..fb.dimensions.width as i64 {
                if current_position.x >= clip.max.x as i64 {
                    break;
                }
                let color = fb.get_pixel(Position { x, y });
                if current_position.x >= clip.min.x && current_position.y >= clip.min.y {
                    self.draw_pixel_unchecked(current_position, color);
                }
                current_position.x += 1;
            }
            current_position.x = position.x;
            current_position.y += 1;
        }
    }

    pub fn draw_fb_apply_alpha(&mut self, fb: &Framebuffer, position: Position) {
        self.draw_fb_apply_alpha_clipped(fb, position, Rect::from_dimensions(self.dimensions))
    }
    pub fn draw_fb_apply_alpha_clipped(
        &mut self,
        fb: &Framebuffer,
        position: Position,
        clip: Rect,
    ) {
        let clip = Rect {
            min: Position::new(clip.min.x.max(0), clip.min.y.max(0)),
            max: Position::new(
                clip.max.x.min(self.dimensions.width as i64),
                clip.max.y.min(self.dimensions.height as i64),
            ),
        };

        let mut current_position = position.clone();

        for y in 0..fb.dimensions.height as i64 {
            if current_position.y >= clip.max.y as i64 {
                return;
            }
            for x in 0..fb.dimensions.width as i64 {
                if current_position.x >= clip.max.x as i64 {
                    break;
                }
                if current_position.x >= clip.min.x && current_position.y >= clip.min.y {
                    let new_color = fb.get_pixel_alpha(Position { x, y });
                    let old_color = self.get_pixel(current_position);
                    let alpha = new_color.1 as f32 / 255.0;
                    let color = Color::new(
                        (old_color.r as f32 * (1.0 - alpha) + new_color.0.r as f32 * alpha) as u8,
                        (old_color.g as f32 * (1.0 - alpha) + new_color.0.g as f32 * alpha) as u8,
                        (old_color.b as f32 * (1.0 - alpha) + new_color.0.b as f32 * alpha) as u8,
                    );
                    self.draw_pixel_unchecked(current_position, color);
                }

                current_position.x += 1;
            }
            current_position.x = position.x;
            current_position.y += 1;
        }
    }

    pub fn draw_fb_scaled(&mut self, fb: &Framebuffer, position: &Position, scale: u32) {
        self.draw_fb_scaled_clipped(fb, position, scale, &Rect::from_dimensions(self.dimensions))
    }
    pub fn draw_fb_scaled_clipped(
        &mut self,
        fb: &Framebuffer,
        position: &Position,
        scale: u32,
        clip: &Rect,
    ) {
        let clip = Rect {
            min: Position::new(clip.min.x.max(0), clip.min.y.max(0)),
            max: Position::new(
                clip.max.x.min(self.dimensions.width as i64),
                clip.max.y.min(self.dimensions.height as i64),
            ),
        };

        let mut current_position = position.clone();

        for y in 0..fb.dimensions.height as i64 {
            if current_position.y >= clip.max.y as i64 {
                return;
            }
            for x in 0..fb.dimensions.width as i64 {
                if current_position.x >= clip.max.x as i64 {
                    break;
                }
                //let new_color = fb.get_pixel_alpha(Position { x, y });
                let color = fb.get_pixel(Position { x, y });
                //let alpha = new_color.1 as f32 / 255.0;
                for y in 0..scale as i64 {
                    for x in 0..scale as i64 {
                        let scale_pos = Position {
                            x: current_position.x + x,
                            y: current_position.y + y,
                        };
                        /*
                        let old_color = self.get_pixel(current_position);
                        let color = Color::new(
                            (old_color.r as f32 * (1.0 - alpha) + new_color.0.r as f32 * alpha)
                                as u8,
                            (old_color.g as f32 * (1.0 - alpha) + new_color.0.g as f32 * alpha)
                                as u8,
                            (old_color.b as f32 * (1.0 - alpha) + new_color.0.b as f32 * alpha)
                                as u8,
                        );
                        */
                        self.draw_pixel(scale_pos, color); // TODO: optimize
                    }
                }
                current_position.x += scale as i64;
            }
            current_position.x = position.x;
            current_position.y += scale as i64;
        }
    }

    pub fn draw_vert_line(&mut self, start: Position, len: i64, color: Color) {
        let start = Position {
            x: start.x.max(0).min(self.dimensions.width as i64),
            y: start.y.max(0).min(self.dimensions.height as i64),
        };

        let end_y = (start.y + len as i64)
            .max(0)
            .min(self.dimensions.height as i64);

        for y in start.y..end_y {
            self.draw_pixel_unchecked(Position { x: start.x, y }, color);
        }
    }

    pub fn draw_line(&mut self, start: Position, end: Position, color: Color) {
        let dx = (end.x - start.x).abs();
        let dy = (end.y - start.y).abs();
        let sx = if start.x < end.x { 1 } else { -1 };
        let sy = if start.y < end.y { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = start.x;
        let mut y = start.y;
        while x != end.x || y != end.y {
            self.draw_pixel(Position { x, y }, color);
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_line_alpha(&mut self, start: Position, end: Position, color: Color, alpha: u8) {
        let dx = (end.x - start.x).abs();
        let dy = (end.y - start.y).abs();
        let sx = if start.x < end.x { 1 } else { -1 };
        let sy = if start.y < end.y { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = start.x;
        let mut y = start.y;
        while x != end.x || y != end.y {
            self.draw_pixel_alpha(Position { x, y }, color, alpha);
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        let mut rect = rect.clone();
        rect.min.x = rect.min.x.max(0).min(self.dimensions.width as i64);
        rect.min.y = rect.min.y.max(0).min(self.dimensions.height as i64);
        rect.max.x = rect.max.x.max(0).min(self.dimensions.width as i64);
        rect.max.y = rect.max.y.max(0).min(self.dimensions.height as i64);

        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                self.draw_pixel_unchecked(Position { x, y }, color);
            }
        }
    }

    pub fn draw_box(&mut self, rect: Rect, color: Color) {
        // TODO: optimize this
        for x in rect.min.x..rect.max.x {
            self.draw_pixel(Position { x, y: rect.min.y }, color);
            self.draw_pixel(
                Position {
                    x,
                    y: rect.max.y - 1,
                },
                color,
            );
        }
        for y in rect.min.y..rect.max.y {
            self.draw_pixel(Position { x: rect.min.x, y }, color);
            self.draw_pixel(
                Position {
                    x: rect.max.x - 1,
                    y,
                },
                color,
            );
        }
    }

    pub fn draw_disc_alpha(&mut self, pos: &Position, radius: u32, color: Color, alpha: u8) {
        use micromath::F32Ext;

        let top_left_position = pos.clone() - Position::new(radius as i64, radius as i64);
        let radius_f = radius as f32;

        for y in 0..radius * 2 {
            for x in 0..radius * 2 {
                let distance = (((x as i64 - radius as i64).pow(2)
                    + (y as i64 - radius as i64).pow(2)) as f32)
                    .sqrt();
                if distance <= radius_f {
                    let color = Color::new(color.r, color.g, color.b);
                    self.draw_pixel_alpha(
                        Position {
                            x: top_left_position.x + x as i64,
                            y: top_left_position.y + y as i64,
                        },
                        color,
                        alpha,
                    );
                }
            }
        }
    }

    pub fn draw_disc(&mut self, pos: Position, radius: u32, color: Color) {
        use micromath::F32Ext;

        let top_left_position = pos.clone() - Position::new(radius as i64, radius as i64);
        let radius_f = radius as f32;

        for y in 0..radius * 2 {
            for x in 0..radius * 2 {
                let distance = (((x as i64 - radius as i64).pow(2)
                    + (y as i64 - radius as i64).pow(2)) as f32)
                    .sqrt();
                if distance <= radius_f {
                    self.draw_pixel(
                        Position {
                            x: top_left_position.x + x as i64,
                            y: top_left_position.y + y as i64,
                        },
                        color,
                    );
                }
            }
        }
    }

    pub fn draw_img(&mut self, image: &Image, position: Position) {
        let clip = Rect::from_dimensions(self.dimensions);

        match &image.data {
            ImageFormat::RGB { .. } => {
                self.draw_img_rgb_clipped(image, position, clip);
            }
            ImageFormat::Bitmap { .. } => self.draw_img_bitmap_clipped(image, position, clip),
        }
    }

    fn draw_img_rgb_clipped(&mut self, image: &Image, position: Position, clip: Rect) {
        let (mut image_data, alpha_val) = match &image.data {
            ImageFormat::RGB { data, alpha_val } => (data.iter(), alpha_val),
            _ => panic!("Invalid image format"),
        };

        let dimensions = image.dimensions();

        if position.y >= self.dimensions.height as i64 || position.x >= self.dimensions.width as i64
        {
            return;
        }

        let clip = Rect {
            min: Position::new(clip.min.x.max(0), clip.min.y.max(0)),
            max: Position::new(
                clip.max.x.min(self.dimensions.width as i64),
                clip.max.y.min(self.dimensions.height as i64),
            ),
        };

        let mut current_position = position.clone();

        for _y in 0..dimensions.height {
            if current_position.y >= clip.max.y as i64 {
                return;
            }
            for x in 0..dimensions.width {
                if current_position.x >= clip.max.x as i64 {
                    let skip = (dimensions.width - x) as usize;
                    image_data.nth(skip * 3 - 1);

                    break;
                }

                let color = Color::new(
                    *image_data.next().unwrap_or(&0),
                    *image_data.next().unwrap_or(&0),
                    *image_data.next().unwrap_or(&0),
                );
                if current_position.x >= clip.min.x && current_position.y >= clip.min.y {
                    let skip_alpha = if let Some(alpha_val) = alpha_val {
                        color == *alpha_val
                    } else {
                        false
                    };

                    if !skip_alpha {
                        self.draw_pixel_unchecked(current_position, color);
                    }
                }

                current_position.x += 1;
            }

            current_position.x = position.x;
            current_position.y += 1;
        }
    }

    fn draw_img_bitmap_clipped(&mut self, image: &Image, position: Position, clip: Rect) {
        let (image_data, color) = match &image.data {
            ImageFormat::Bitmap { data, color } => (data, color),
            _ => panic!("Invalid image format"),
        };

        let dimensions = image.dimensions();
        let bytes_per_row =
            dimensions.width as usize / 8 + if dimensions.width % 8 != 0 { 1 } else { 0 };

        if position.y >= self.dimensions.height as i64 || position.x >= self.dimensions.width as i64
        {
            return;
        }

        let clip = Rect {
            min: Position::new(clip.min.x.max(0), clip.min.y.max(0)),
            max: Position::new(
                clip.max.x.min(self.dimensions.width as i64),
                clip.max.y.min(self.dimensions.height as i64),
            ),
        };

        let mut current_position = position.clone();

        for y in 0..dimensions.height as i64 {
            if current_position.y >= clip.max.y as i64 {
                return;
            }
            for x in 0..dimensions.width as i64 {
                if current_position.x >= clip.max.x as i64 {
                    break;
                }

                if current_position.x >= clip.min.x && current_position.y >= clip.min.y {
                    let byte_offset = (y * bytes_per_row as i64 + x / 8) as usize;
                    let bit_offset = x % 8;
                    let bit_offset = 7 - bit_offset;
                    if image_data[byte_offset] & (1 << bit_offset) == 0 {
                        self.draw_pixel_unchecked(current_position, *color);
                    }
                }

                current_position.x += 1;
            }

            current_position.x = position.x;
            current_position.y += 1;
        }
    }

    pub fn draw_char<F: Font>(&mut self, color: Color, character: char, position: &Position) {
        if let Some(char) = F::get_char(character) {
            let char = Character::from_raw(char);
            let mut bit_offset = 0;
            let mut byte_offset = 0;

            for y in 0..char.height {
                for x in 0..char.width {
                    let byte = char.data.get(byte_offset).unwrap_or(&1);
                    if byte & (1 << bit_offset) == 0 {
                        self.draw_pixel(
                            Position {
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

    #[inline(always)]
    pub fn draw_str<F: Font>(&mut self, color: Color, string: &str, position: Position) {
        self.draw_str_clipped::<F>(
            color,
            string,
            position,
            Rect::from_dimensions(self.dimensions),
        );
    }

    pub fn draw_str_clipped<F: Font>(
        &mut self,
        color: Color,
        string: &str,
        position: Position,
        clip: Rect,
    ) {
        if position.y >= self.dimensions.height as i64 || position.x >= self.dimensions.width as i64
        {
            return;
        }

        let clip = Rect {
            min: Position::new(clip.min.x.max(0), clip.min.y.max(0)),
            max: Position::new(
                clip.max.x.min(self.dimensions.width as i64),
                clip.max.y.min(self.dimensions.height as i64),
            ),
        };

        let chars = string
            .chars()
            .filter_map(|c| F::get_char(c).map(|c| Character::from_raw(c)))
            .collect::<Vec<_>>();

        let mut current_position = position.clone();

        for y in 0..F::CHAR_HEIGHT as usize {
            if current_position.y >= clip.max.y as i64 {
                return;
            }
            'inner: for c in &chars {
                for x in 0..c.width {
                    if current_position.x >= clip.max.x as i64 {
                        break 'inner;
                    }

                    let c_bit_offset = (y * c.width + x) as usize;
                    let c_byte_offset = c_bit_offset / 8;
                    let c_bit_offset = c_bit_offset % 8;

                    let byte = c.data.get(c_byte_offset).unwrap_or(&1);

                    if byte & (1 << c_bit_offset) == 0 {
                        if current_position.x > clip.min.x && current_position.y > clip.min.y {
                            self.draw_pixel_unchecked(current_position, color);
                        }
                    }

                    current_position.x += 1;
                }
            }

            current_position.x = position.x;
            current_position.y += 1;
        }
    }

    #[inline(always)]
    pub fn draw_pixel(&mut self, position: Position, color: Color) {
        if position.x >= self.dimensions.width.into()
            || position.y >= self.dimensions.height.into()
            || position.x < 0
            || position.y < 0
        {
            return;
        }

        self.draw_pixel_unchecked(position, color);
    }

    #[inline(always)]
    fn draw_pixel_unchecked(&mut self, position: Position, color: Color) {
        let byte_offset = self.pos_to_offset(position) as usize;

        let pixel_bytes = &mut self.buffer[byte_offset..];
        pixel_bytes[self.format.r_position] = color.r;
        pixel_bytes[self.format.g_position] = color.g;
        pixel_bytes[self.format.b_position] = color.b;
    }

    #[inline(always)]
    pub fn draw_pixel_alpha(&mut self, position: Position, color: Color, alpha: u8) {
        if position.x >= self.dimensions.width.into()
            || position.y >= self.dimensions.height.into()
            || position.x < 0
            || position.y < 0
        {
            return;
        }

        self.draw_pixel_alpha_unchecked(position, color, alpha);
    }

    #[inline(always)]
    fn draw_pixel_alpha_unchecked(&mut self, position: Position, color: Color, alpha: u8) {
        let byte_offset = self.pos_to_offset(position) as usize;

        let pixel_bytes = &mut self.buffer[byte_offset..];
        pixel_bytes[self.format.r_position] = color.r;
        pixel_bytes[self.format.g_position] = color.g;
        pixel_bytes[self.format.b_position] = color.b;
        if let Some(a_position) = self.format.a_position {
            pixel_bytes[a_position] = alpha;
        }
    }
}
