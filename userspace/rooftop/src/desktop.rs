use monos_gfx::{
    font::Cozette,
    ui::{Direction, MarginMode, UIContext, UIFrame},
    Framebuffer, Image, Input, Rect,
};

pub struct Desktop {
    bounds: Rect,
    ui: UIFrame,
    entries: Vec<DesktopEntry>,
}

#[derive(Debug)]
struct DesktopEntry {
    name: String,
    icon: Image,
    action: EntryAction,
}

#[derive(Debug)]
enum EntryAction {
    Open { bin: PathBuf, arg: String },
}

impl EntryAction {
    fn execute(&self) {
        match self {
            Self::Open { bin, arg: _ } => {
                // TODO: pass arg
                match syscall::spawn(bin /* arg*/) {
                    None => {
                        println!("Failed to spawn process");
                    }
                    _ => {}
                }
            }
        };
    }
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
            ui.margin(MarginMode::AtLeast(50));
            for entry in &self.entries {
                if ui.img_button(&entry.icon).clicked {
                    entry.action.execute();
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
                    entry.action.execute();
                };
                ui.label::<Cozette>(&entry.name);
            }
        })
    }

    fn update_entries(&mut self) {
        let entries = syscall::list("home/desktop");

        self.entries.clear();
        entries
            .iter()
            .filter_map(|entry| File::open(entry))
            .filter_map(|file| file.read_to_string().ok())
            .for_each(|entry| {
                let mut name = None;
                let mut icon = None;
                let mut open = None;
                let mut args = None;

                for line in entry.lines() {
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

                    let entry = DesktopEntry {
                        name,
                        icon,
                        action: EntryAction::Open {
                            bin: open,
                            arg: args,
                        },
                    };

                    self.entries.push(entry);
                }
            })
    }
}

