use monos_gfx::{
    font::Cozette,
    ui::{Direction, MarginMode, PaddingMode, UIFrame},
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
    Open(PathBuf),
    OpenWith { bin: PathBuf, arg: String },
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
                    dbg!(entry);
                    syscall::spawn("bin/hello_world");
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
            .filter_map(|file| {
                let mut data = vec![0; 255]; // TODO: stat file to get size
                let len = file.read(&mut data);
                data.truncate(len);
                String::from_utf8(data).ok()
            })
            .for_each(|entry| {
                let mut name = None;
                let mut icon = None;
                let mut open = None;
                let mut open_with = None;

                for line in entry.lines() {
                    let (key, value) = line.split_once('=').unwrap();
                    match key {
                        "name" => name = Some(value),
                        "icon" => {
                            icon = { File::open(value).and_then(|file| Image::from_ppm(&file)) }
                        }
                        "open" => open = Some(value),
                        "open_with" => open_with = Some(value),
                        _ => {}
                    }
                }

                if let (Some(name), Some(icon), Some(open)) = (name, icon, open) {
                    let name = name.to_string();
                    let open = String::from(open);
                    let open_with = open_with.map(PathBuf::from);

                    let entry = if let Some(open_with) = open_with {
                        DesktopEntry {
                            name,
                            icon,
                            action: EntryAction::OpenWith {
                                bin: open_with,
                                arg: open,
                            },
                        }
                    } else {
                        DesktopEntry {
                            name,
                            icon,
                            action: EntryAction::Open(PathBuf::from(open)),
                        }
                    };

                    self.entries.push(entry);
                }
            })
    }
}
