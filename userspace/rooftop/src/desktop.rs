use monos_gfx::{input::Input, Color, Framebuffer, PaintFramebuffer, Position, Rect};

mod desktop_entries;
use desktop_entries::DesktopEntries;

pub struct Desktop<'fb> {
    clear_fb: Framebuffer<'fb>,
    paint_fb: PaintFramebuffer<'fb>,

    taskbar: monos_gfx::Image,
    entries: DesktopEntries,

    needs_redraw: bool,
}

impl<'fb> Desktop<'fb> {
    pub fn paint(&mut self) -> &mut PaintFramebuffer<'fb> {
        self.needs_redraw = true;
        &mut self.paint_fb
    }

    pub fn new(
        main_fb: &Framebuffer,
        clear_fb_buf: &'fb mut Vec<u8>,
        paint_fb_buf: &'fb mut Vec<u8>,
    ) -> Self {
        clear_fb_buf.resize(main_fb.buffer().len(), 0);
        let clear_fb =
            Framebuffer::new(clear_fb_buf, main_fb.dimensions(), main_fb.format().clone());

        paint_fb_buf.resize(main_fb.buffer().len(), 0);
        let mut paint_fb =
            Framebuffer::new(paint_fb_buf, main_fb.dimensions(), main_fb.format().clone());
        paint_fb.draw_rect(
            Rect::from_dimensions(main_fb.dimensions()),
            Color::new(55, 54, 61),
        );
        let paint_fb = PaintFramebuffer::new(paint_fb);

        let taskbar = File::open("data/task.ppm").expect("failed to load image data");
        let taskbar = monos_gfx::Image::from_ppm(&taskbar).expect("failed to parse image data");

        let desktop_rect = Rect::new(
            Position::new(0, 0),
            Position::new(
                main_fb.dimensions().width as i64,
                main_fb.dimensions().height as i64 - taskbar.dimensions().height as i64,
            ),
        );
        let entries = DesktopEntries::new(desktop_rect);

        let mut desktop = Self {
            clear_fb,
            paint_fb,

            taskbar,
            entries,

            needs_redraw: false,
        };

        desktop.rebuild();

        desktop
    }

    fn rebuild(&mut self) {
        self.clear_fb.clear_with(&self.paint_fb);

        self.clear_fb.draw_img(
            &self.taskbar,
            Position::new(
                0,
                (self.clear_fb.dimensions().height - self.taskbar.dimensions().height) as i64,
            ),
        );

        self.entries.draw(&mut self.clear_fb, &mut Input::default());
    }

    pub fn update(&mut self, input: &mut Input) -> bool {
        self.entries.layout(input);

        if self.needs_redraw {
            self.rebuild();
            self.needs_redraw = false;
            true
        } else {
            false
        }
    }
}

impl<'fb> core::ops::Deref for Desktop<'fb> {
    type Target = Framebuffer<'fb>;

    fn deref(&self) -> &Self::Target {
        &self.clear_fb
    }
}
