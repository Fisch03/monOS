use minifb::{Window, WindowOptions};
use monoscript::{execute, parse, Interface, ScriptContext, ScriptHook};

mod key;
use key::char_to_key;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

#[derive(Debug)]
struct EmuInterface<'a> {
    windows: Vec<EmuInterfaceWindow>,
    contents: Vec<ScriptHook<'a>>,
    current_window: usize,
    key_hooks: Vec<(char, ScriptHook<'a>)>,
}
#[derive(Debug)]
struct EmuInterfaceWindow {
    buffer: Vec<u32>,

    window: Window,
}
impl<'a> EmuInterface<'a> {
    fn new() -> Self {
        Self {
            windows: Vec::new(),
            contents: Vec::new(),
            key_hooks: Vec::new(),
            current_window: 0,
        }
    }

    fn some_window_open(&self) -> bool {
        self.windows.len() > 0 && self.windows.iter().any(|w| w.window.is_open())
    }

    fn update_windows(&mut self, context: &mut ScriptContext<'a>) {
        let contents = self.contents.drain(..).collect::<Vec<_>>();

        for (window_idx, c) in contents.iter().enumerate() {
            self.current_window = window_idx;

            //let start = std::time::Instant::now();
            c.execute(context, self).expect("failed to render window");
            //println!("rendered in {:?}", start.elapsed());

            let w = &mut self.windows[window_idx];
            w.window
                .update_with_buffer(&w.buffer, WIDTH, HEIGHT)
                .unwrap();

            w.buffer.iter_mut().for_each(|p| *p = 0);
        }

        self.contents.extend(contents);
    }

    fn update_key_hooks(&mut self, context: &mut ScriptContext<'a>) {
        let key_hooks = self.key_hooks.drain(..).collect::<Vec<_>>();

        for (key, hook) in key_hooks.iter() {
            if self.is_key_down(*key) {
                hook.execute(context, self)
                    .expect("failed to execute key hook");
            }
        }

        self.key_hooks.extend(key_hooks);
    }

    fn is_key_down(&self, key: char) -> bool {
        self.windows
            .iter()
            .any(|w| w.window.is_key_down(char_to_key(key)))
    }
}

impl<'a> Interface<'a> for EmuInterface<'a> {
    fn print(&self, message: &str) {
        print!("{}", message);
    }

    fn on_key(&mut self, key: char, content: ScriptHook<'a>) {
        self.key_hooks.push((key, content));
    }

    fn spawn_window(&mut self, content: ScriptHook<'a>) {
        if self.windows.len() >= 10 {
            eprintln!("too many windows!!!");
            return;
        }

        let buffer = vec![0; WIDTH * HEIGHT];
        let mut window =
            Window::new("Monoscript Emu", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

        window.set_target_fps(60);
        self.windows.push(EmuInterfaceWindow { buffer, window });
        self.contents.push(content);
    }

    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize) {
        if let Some(window) = self.windows.get_mut(self.current_window) {
            for i in 0..w {
                for j in 0..h {
                    let x = x + i;
                    let y = y + j;
                    if x < WIDTH && y < HEIGHT {
                        window.buffer[y * WIDTH + x] = 0xFFFFFFFF;
                    }
                }
            }
        }
    }
}

pub fn run_script(script: &str) -> Result<(), ()> {
    let start = std::time::Instant::now();
    let parsed = parse(&script).expect("failed to parse script");
    println!("parsed in {:?}", start.elapsed());

    let mut interface = EmuInterface::new();

    let start = std::time::Instant::now();
    let mut context = execute(parsed, &mut interface).expect("failed to execute script");
    println!("executed in {:?}", start.elapsed());

    while interface.some_window_open() {
        interface.update_key_hooks(&mut context);
        interface.update_windows(&mut context);
    }

    Ok(())
}
