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
    recv_handle: PartialReceiveChannelHandle,
    drag_start: Option<Position>,
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
            drag_start: None,
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
                chunk.update_frequency = UpdateFrequency::default();

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

        let focused_window = self.windows.len() - 1;
        if input.mouse.left_button.pressed {
            if let Some(drag_start) = self.drag_start {
                // drag
                let window = &mut self.windows[focused_window];
                let chunk = window.chunk.as_ref().unwrap();

                let window_rect = Rect::new(window.pos, window.pos + chunk.dimensions);

                let header_rect = Rect::new(
                    Position::new(window_rect.min.x, window_rect.min.y - 16),
                    Position::new(window_rect.max.x, window_rect.min.y),
                );

                let full_rect = Rect::new(header_rect.min, window_rect.max).grow(1);

                window.pos += input.mouse.position - drag_start;
                self.drag_start = Some(input.mouse.position);

                fb.clear_region(&full_rect, &clear_fb);
            }
        } else {
            self.drag_start = None;
        }

        if input.mouse.left_button.clicked {
            let new_focused_window =
                self.windows
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(i, window)| {
                        let chunk = window.chunk.as_ref().unwrap();

                        let window_rect = Rect::new(window.pos, window.pos + chunk.dimensions);

                        let header_rect = Rect::new(
                            Position::new(window_rect.min.x, window_rect.min.y - 16),
                            Position::new(window_rect.max.x, window_rect.min.y),
                        );

                        if header_rect.contains(input.mouse.position) {
                            return Some(i);
                        }

                        None
                    });

            if let Some(new_focused_window) = new_focused_window {
                // drag start + focus
                self.windows.swap(new_focused_window, focused_window);
                self.drag_start = Some(input.mouse.position);
            }
        }

        for (i, window) in self.windows.iter_mut().enumerate() {
            let focused = i == focused_window;
            let mut closed = false;

            let mut chunk = window.chunk.as_mut().unwrap();
            window.title = String::from(chunk.title());

            let window_rect = Rect::new(window.pos, window.pos + chunk.dimensions);

            let header_rect = Rect::new(
                Position::new(window_rect.min.x, window_rect.min.y - 16),
                Position::new(window_rect.max.x, window_rect.min.y),
            );
            let full_rect = Rect::new(header_rect.min, window_rect.max).grow(1);
            let bg_color = if focused {
                Color::new(22, 22, 22)
            } else {
                Color::new(44, 44, 44)
            };
            fb.draw_rect(header_rect, bg_color);
            fb.draw_box(full_rect, bg_color);

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
                self.drag_start = None;

                fb.clear_region(&full_rect, &clear_fb);
            } else {
                let should_send = match chunk.update_frequency {
                    UpdateFrequency::Always => true,
                    UpdateFrequency::OnInput => input.any(),
                    UpdateFrequency::Manual => false,
                }; //TODO: manual updates

                if should_send {
                    let mut mouse = input.mouse.clone();
                    mouse.position -= window_rect.min;
                    chunk.mouse = mouse;

                    let keyboard_src = &input.keyboard.keys[..key_amt];
                    let keyboard_dest = &mut chunk.keyboard[..keyboard_src.len()];
                    keyboard_dest.clone_from_slice(keyboard_src);
                    chunk.keyboard_len = keyboard_src.len() as u8;

                    chunk.focused = focused;

                    window
                        .target_handle
                        .send(WindowServerMessage::RequestRender(
                            window.chunk.take().unwrap(),
                        ));
                }
            }
        }

        self.windows.retain(|w| !closed_windows.contains(&w.id));
    }

    pub fn draw_window_list(
        &mut self,
        fb: &mut Framebuffer,
        rect: Rect,
        input: &mut Input,
        clear_fb: &Framebuffer,
    ) {
        fb.clear_region(&rect, clear_fb);

        let mut new_focused_window = None;

        let mut ui = UIFrame::new_stateless(Direction::LeftToRight);
        ui.draw_frame(fb, rect, input, |ui| {
            ui.margin(MarginMode::Grow);

            let mut names = self
                .windows
                .iter()
                .enumerate()
                .map(|(i, w)| (i, w.id, &w.title))
                .collect::<Vec<_>>();
            names.sort_by(|a, b| a.1.cmp(&b.1));

            for (i, _, name) in names {
                if ui.button::<font::Cozette>(name).clicked {
                    new_focused_window = Some(i);
                }
            }
        });

        if let Some(new_focused_window) = new_focused_window {
            let focused_window = self.windows.len() - 1;
            self.windows.swap(new_focused_window, focused_window);
        }
    }
}
