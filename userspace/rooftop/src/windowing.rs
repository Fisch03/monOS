use monos_gfx::{
    font,
    ui::{Direction, MarginMode, UIFrame},
    Color, Dimension, Framebuffer, FramebufferFormat, Image, Input, Position, Rect,
};
use monos_std::messaging::*;

const SCREEN_RECT: Rect = Rect::new(Position::new(0, 0), Position::new(640, 480));

pub struct WindowServer {
    windows: Vec<Window>,
    close_button: Image,
    window_id: u64,
    recv_handle: PartialReceiveChannelHandle,
}

#[derive(Debug)]
pub struct Window {
    id: u64,
    title: String,
    // icon: Image,
    pos: Position,
    chunk: Option<MemoryChunk<WindowChunk>>,
    target_handle: ChannelHandle,
}

pub struct WindowChunk {
    id: u64,
    dimensions: Dimension,
    title: [u8; 32],
    title_len: u8,
    data: [u8; 640 * 480 * 3],
}

impl WindowChunk {
    pub fn title(&self) -> &str {
        core::str::from_utf8(&self.title).unwrap_or("<empty>")
    }

    pub fn set_title(&mut self, title: &str) {
        let title_slice = self.title[0..title.len()].as_mut();
        title_slice.copy_from_slice(title.as_bytes());
        self.title_len = title.len() as u8;
    }

    pub fn fb(&mut self) -> Framebuffer {
        Framebuffer::new(
            &mut self.data[..self.dimensions.width as usize * self.dimensions.height as usize * 3],
            self.dimensions,
            FramebufferFormat {
                bytes_per_pixel: 3,
                stride: self.dimensions.width as u64,
                r_position: 0,
                g_position: 1,
                b_position: 2,
                a_position: None,
            },
        )
    }
}

impl core::fmt::Debug for WindowChunk {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WindowChunk")
            .field("id", &self.id)
            .field("title", &self.title())
            .finish()
    }
}

impl WindowServer {
    pub fn new(port: &str) -> Self {
        let recv_handle = syscall::serve(port).unwrap();

        let close_button = File::open("data/close.ppm").unwrap();
        let close_button = Image::from_ppm(&close_button).unwrap();

        WindowServer {
            windows: Vec::new(),
            close_button,
            window_id: 0,
            recv_handle,
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
            WindowClientMessage::CreateWindow { dimensions } => {
                let id = self.window_id;
                self.window_id += 1;

                let rect = Rect::centered_in(SCREEN_RECT, dimensions);

                let mut chunk = syscall::request_chunk::<WindowChunk>().unwrap();
                chunk.id = id;
                chunk.dimensions = dimensions;
                chunk.title = [0; 32];

                let target_handle = ChannelHandle::from_parts(sender, self.recv_handle);

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

    pub fn draw(&mut self, fb: &mut Framebuffer, input: &mut Input) {
        for window in &mut self.windows {
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
                    println!("todo: close window {}", window.id);
                }
            });

            window
                .target_handle
                .send(WindowServerMessage::RequestRender(chunk));
        }
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

// sent from rooftop to window clients
#[derive(Debug)]
pub enum WindowServerMessage {
    RequestClose { id: u64 },
    RequestRender(MemoryChunk<WindowChunk>),
}

impl MessageData for WindowServerMessage {
    fn into_message(self) -> MessageType {
        match self {
            WindowServerMessage::RequestClose { id } => MessageType::Scalar(0, id, 0, 0),
            WindowServerMessage::RequestRender(chunk) => chunk.as_message(0, 0),
        }
    }

    unsafe fn from_message(msg: GenericMessage) -> Option<Self> {
        match msg.data {
            MessageType::Scalar(0, id, _, _) => Some(WindowServerMessage::RequestClose { id }),
            MessageType::Chunk { data: (0, _), .. } => {
                let chunk = msg.data.as_chunk::<WindowChunk>();
                chunk.map(|chunk| WindowServerMessage::RequestRender(chunk))
            }
            _ => None,
        }
    }
}

// sent from window clients to server
#[derive(Debug)]
pub enum WindowClientMessage {
    CreateWindow { dimensions: Dimension },
    SubmitRender(MemoryChunk<WindowChunk>),
}

impl MessageData for WindowClientMessage {
    fn into_message(self) -> MessageType {
        match self {
            WindowClientMessage::CreateWindow { dimensions } => {
                MessageType::Scalar(0, dimensions.width as u64, dimensions.height as u64, 0)
            }
            WindowClientMessage::SubmitRender(chunk) => chunk.as_message(0, 0),
        }
    }

    unsafe fn from_message(msg: GenericMessage) -> Option<Self> {
        match msg.data {
            MessageType::Scalar(0, width, height, _) => Some(WindowClientMessage::CreateWindow {
                dimensions: Dimension::new(width as u32, height as u32),
            }),
            MessageType::Chunk { data: (0, _), .. } => {
                let chunk = msg.data.as_chunk::<WindowChunk>();
                chunk.map(|chunk| WindowClientMessage::SubmitRender(chunk))
            }
            _ => None,
        }
    }
}
