use monos_gfx::{
    font::Cozette,
    ui::{Direction, UIFrame},
    Framebuffer, Image, Input, Rect,
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
        let mut desktop = Self {
            bounds,
            ui: UIFrame::new(Direction::TopToBottom),
            entries: Vec::new(),
        };

        desktop.update_entries();

        desktop
    }

    pub fn draw(&mut self, fb: &mut Framebuffer, input: &mut Input) {
        self.ui.draw_frame(fb, self.bounds, input, |ui| {
            for entry in &self.entries {
                ui.img_button(&entry.icon);
                ui.label::<Cozette>(&entry.name);
            }
        })
    }

    fn update_entries(&mut self) {
        let entries = syscall::list("home/desktop");

        entries
            .iter()
            .filter_map(|entry| File::open(entry))
            .filter_map(|file| {
                let mut data = vec![0; 255]; // TODO: stat file to get size
                let len = file.read(&mut data);
                data.truncate(len);
                String::from_utf8(data).ok()
            })
            .for_each(|entry| {
                println!("{}\n\n", entry);
            })
    }
}
