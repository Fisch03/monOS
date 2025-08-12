use monos_gfx::{
    font::Cozette,
    ui::{Direction, MarginMode, UIFrame},
    Framebuffer, Image, Input, Rect,
};

pub struct DesktopEntries {
    bounds: Rect,
    ui: UIFrame,
    entries: Vec<DesktopEntry>,
}

#[derive(Debug)]
struct DesktopEntry {
    name: String,
    icon: Image,
    bin: PathBuf,
    arg: String,
}

impl DesktopEntry {
    fn execute(&self) {
        match syscall::spawn_with_args(&self.bin, &self.arg) {
            None => {
                println!("Failed to spawn process");
            }
            _ => {}
        }
    }
}

impl DesktopEntries {
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
            ui.margin(MarginMode::AtLeast(50));
            for entry in &self.entries {
                if ui.img_button(&entry.icon).clicked {
                    entry.execute();
                };
                ui.label::<Cozette>(&entry.name);
            }
        })
    }

    pub fn layout(&mut self, input: &mut Input) {
        self.ui.layout_frame(self.bounds, input, |ui| {
            ui.margin(MarginMode::AtLeast(50));
            for entry in &self.entries {
                if ui.img_button(&entry.icon).clicked {
                    entry.execute();
                };
                ui.label::<Cozette>(&entry.name);
            }
        })
    }

    fn parse_entry_file(file: File) -> Option<DesktopEntry> {
        let content = file.read_to_string().ok()?;

        let mut name = None;
        let mut icon = None;
        let mut open = None;
        let mut args = None;

        for line in content.lines() {
            let (key, value) = line.split_once('=').unwrap();
            match key {
                "name" => name = Some(value),
                "icon" => icon = File::open(value).and_then(|file| Image::from_ppm(&file)),
                "open" => open = Some(value),
                "args" => args = Some(value),
                _ => {}
            }
        }

        if let (Some(name), Some(icon), Some(open)) = (name, icon, open) {
            let name = name.to_string();
            let open = PathBuf::from(open);
            let args = args.map(String::from).unwrap_or_default();

            Some(DesktopEntry {
                name,
                icon,
                bin: open,
                arg: args,
            })
        } else {
            None
        }
    }

    fn update_entries(&mut self) {
        let entries = syscall::list("home/desktop");

        self.entries.clear();
        self.entries.extend(
            entries
                .iter()
                .filter_map(|path| File::open(path).map(|f| (f, Path::from(path))))
                .filter_map(|(file, path)| match path.extension() {
                    Some("de") => Self::parse_entry_file(file),
                    Some("ms") => Some(DesktopEntry {
                        name: path.file_name()?.to_string(),
                        icon: File::open("data/icons/ms.ppm")
                            .and_then(|file| Image::from_ppm(&file))
                            .expect("failed to load ms icon"),

                        bin: PathBuf::from("bin/terminal"),
                        arg: path.to_string(),
                    }),

                    _ => {
                        println!("skipping unrecognized desktop file: {}", path);
                        None
                    }
                }),
        );
    }
}
