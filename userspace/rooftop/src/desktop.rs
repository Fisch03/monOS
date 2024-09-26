use monos_gfx::{
    ui::{Direction, UIFrame},
    Framebuffer, Image, Rect,
};

pub struct Desktop {
    bounds: Rect,
    ui: UIFrame,
    entries: Vec<DesktopEntry>,
}

struct DesktopEntry {
    name: String,
    icon: Image,
    open_with: Option<PathBuf>,
}

impl Desktop {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            ui: UIFrame::new(Direction::TopToBottom),
            entries: Vec::new(),
        }
    }

    pub fn draw(&mut self, fb: &mut Framebuffer) {
        //self.update_entries();
        //self.ui.draw(fb, self.bounds);
    }
}
