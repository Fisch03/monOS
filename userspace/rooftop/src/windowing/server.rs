use monos_gfx::{
    font,
    ui::{Direction, MarginMode, UIFrame},
    Color, Framebuffer, Image, Input, Position, Rect,
};

use super::*;

const SCREEN_RECT: Rect = Rect::new(Position::new(0, 0), Position::new(640, 480));

#[derive(Debug)]
pub struct Window {
    id: u64,
    title: String,
    // icon: Image,
    pos: Position,
    chunk: Option<MemoryChunk<WindowChunk>>,
    target_handle: ChannelHandle,
}

pub struct WindowServer {
    windows: Vec<Window>,
    close_button: Image,
    window_id: u64,
    focused_window: Option<u64>,
    recv_handle: PartialReceiveChannelHandle,
}

impl WindowServer {
    pub fn new(port: &str) -> Self {
        let recv_handle = syscall::serve(port).unwrap();

        let close_button = File::open("data/close.ppm").unwrap();
        let close_button = Image::from_ppm(&close_button).unwrap();

        WindowServer {
            windows: Vec::new(),
            close_button,
            window_id: 1, // window id 0 is reserved for windows with an unknown id
            recv_handle,
            focused_window: None,
        }
    }

    // safety: msg must be a WindowServerMessage
    pub unsafe fn handle_message(&mut self, msg: GenericMessage) {
        let sender = msg.sender;
        let msg = unsafe { WindowClientMessage::from_message(msg) };
        let msg = match msg {
            Some(msg) => msg,
            None => return,
        };

        match msg {
            WindowClientMessage::CreateWindow {
                dimensions,
                creation_id,
            } => {
                let id = self.window_id;
                self.window_id += 1;

                let rect = Rect::centered_in(SCREEN_RECT, dimensions);

                let mut chunk = syscall::request_chunk::<WindowChunk>().unwrap();
                chunk.id = id;
                chunk.dimensions = dimensions;
                chunk.title = [0; 32];
                chunk.keyboard_len = 0;

                let target_handle = ChannelHandle::from_parts(sender, self.recv_handle);

                target_handle.send(WindowServerMessage::ConfirmCreation { id, creation_id });
                target_handle.send(WindowServerMessage::RequestRender(chunk));

                self.windows.push(Window {
                    id,
                    title: format!("window {}", id),
                    pos: rect.min,
                    chunk: None,
                    target_handle,
                });

                self.focused_window = Some(id);

                println!(
                    "created window {} with dimensions {}x{}",
                    id, dimensions.width, dimensions.height
                );

                // syscall::send(
                //     sender,
                //     WindowServerMessage::WindowCreated { id }.into_message(),
                // );
            }

            WindowClientMessage::SubmitRender(chunk) => {
                let window = self.windows.iter_mut().find(|w| w.id == chunk.id);
                if let Some(window) = window {
                    window.chunk = Some(chunk);
                }
            }
        }
    }

    pub fn ready_to_render(&self) -> bool {
        !self.windows.iter().any(|w| w.chunk.is_none())
    }

    pub fn draw(&mut self, fb: &mut Framebuffer, input: &mut Input, clear_fb: &Framebuffer) {
        let mut closed_windows = Vec::new();

        let mut key_amt = input.keyboard.keys.len();
        if key_amt > 6 {
            println!(
                "warning: dropping {} keyboard events",
                input.keyboard.keys.len() - 6
            );
            key_amt = 6;
        }

        for window in &mut self.windows {
            let mut closed = false;

            let mut chunk = window.chunk.take().unwrap();
            window.title = String::from(chunk.title());

            let window_rect = Rect::new(window.pos, window.pos + chunk.dimensions);

            let header_rect = Rect::new(
                Position::new(window_rect.min.x, window_rect.min.y - 16),
                Position::new(window_rect.max.x, window_rect.min.y),
            );
            let full_rect = Rect::new(header_rect.min, window_rect.max);
            fb.draw_rect(header_rect, Color::new(22, 22, 22));
            fb.draw_box(full_rect.grow(1), Color::new(22, 22, 22));

            fb.draw_fb(&chunk.fb(), window_rect.min);

            let mut title_ui = UIFrame::new_stateless(Direction::LeftToRight);
            title_ui.draw_frame(fb, header_rect, input, |ui| {
                ui.margin(MarginMode::Grow);
                ui.label::<font::Cozette>(&window.title);
            });

            let mut btn_ui = UIFrame::new_stateless(Direction::RightToLeft);
            btn_ui.draw_frame(fb, header_rect, input, |ui| {
                ui.margin(MarginMode::Grow);

                if ui.img_button(&self.close_button).clicked {
                    closed = true;
                }
            });

            if closed {
                closed_windows.push(window.id);
                window
                    .target_handle
                    .send(WindowServerMessage::RequestClose { id: window.id });

                fb.clear_region(&full_rect.grow(1), &clear_fb);
            } else {
                let mut mouse = input.mouse.clone();
                mouse.position -= window_rect.min;
                chunk.mouse = mouse;

                let keyboard_src = &input.keyboard.keys[..key_amt];
                let keyboard_dest = &mut chunk.keyboard[..keyboard_src.len()];
                keyboard_dest.clone_from_slice(keyboard_src);
                chunk.keyboard_len = keyboard_src.len() as u8;

                chunk.focused = self.focused_window == Some(window.id);

                window
                    .target_handle
                    .send(WindowServerMessage::RequestRender(chunk));
            }
        }

        self.windows.retain(|w| !closed_windows.contains(&w.id));
    }

    pub fn draw_window_list(&self, fb: &mut Framebuffer, rect: Rect, input: &mut Input) {
        let mut ui = UIFrame::new(Direction::LeftToRight);
        ui.draw_frame(fb, rect, input, |ui| {
            ui.margin(MarginMode::Grow);

            for window in &self.windows {
                ui.button::<font::Cozette>(&window.title);
            }
        });
    }
}
