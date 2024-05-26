use minifb::{Window, WindowOptions};
use monoscript::{execute, interpret, parse, Interface};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

#[derive(Debug)]
struct EmuInterface {
    window: Option<EmuInterfaceWindow>,
}
#[derive(Debug)]
struct EmuInterfaceWindow {
    buffer: Vec<u32>,
    window: Window,
}
impl EmuInterface {
    fn new() -> Self {
        Self { window: None }
    }

    fn create_window(&mut self) {
        let buffer = vec![0; WIDTH * HEIGHT];
        let mut window =
            Window::new("Monoscript Emu", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

        window.set_target_fps(60);
        self.window = Some(EmuInterfaceWindow { buffer, window });
    }

    fn window_is_open(&self) -> bool {
        self.window.as_ref().map_or(false, |w| w.window.is_open())
    }

    fn update_window(&mut self) {
        if let Some(window) = &mut self.window {
            window
                .window
                .update_with_buffer(&window.buffer, WIDTH, HEIGHT)
                .unwrap();
            window.buffer.iter_mut().for_each(|p| *p = 0);
        }
    }
}

impl Interface for EmuInterface {
    fn print(&self, message: &str) {
        print!("{}", message);
    }

    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize) {
        if let Some(window) = &mut self.window {
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
    let interpreted = interpret(parsed).expect("failed to interpret script");
    println!("Parsed and interpreted in {:?}", start.elapsed());

    let mut interface = EmuInterface::new();
    if let Some(mut persistent_code) =
        execute(interpreted, &mut interface).expect("failed to execute script")
    {
        if persistent_code.wants_window() {
            interface.create_window();
            while interface.window_is_open() {
                persistent_code.on_window(&mut interface);
                interface.update_window();
            }
        }
    }

    Ok(())
}
