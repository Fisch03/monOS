use monos_gfx::{
    input::{KeyEvent, MouseInput},
    Dimension, Framebuffer, FramebufferFormat,
};
use monos_std::messaging::*;

pub mod client;
pub mod server;

pub struct WindowChunk {
    id: u64,
    dimensions: Dimension,
    title: [u8; 32],
    title_len: u8,
    update_frequency: UpdateFrequency,
    focused: bool,
    mouse: MouseInput,
    keyboard: [KeyEvent; 6],
    keyboard_len: u8,
    data: [u8; 640 * 480 * 3],
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateFrequency {
    Manual,
    Always,
    OnInput,
}

impl core::default::Default for UpdateFrequency {
    fn default() -> Self {
        UpdateFrequency::OnInput
    }
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

    pub fn keys(&self) -> &[KeyEvent] {
        &self.keyboard[..self.keyboard_len as usize]
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

// sent from rooftop to window clients
#[derive(Debug)]
pub enum WindowServerMessage {
    RequestClose { id: u64 },
    ConfirmCreation { id: u64, creation_id: u64 },
    RequestRender(MemoryChunk<WindowChunk>),
}

impl MessageData for WindowServerMessage {
    fn into_message(self) -> MessageType {
        match self {
            WindowServerMessage::RequestClose { id } => MessageType::Scalar(0, id, 0, 0),
            WindowServerMessage::ConfirmCreation { id, creation_id } => {
                MessageType::Scalar(1, id, creation_id, 0)
            }
            WindowServerMessage::RequestRender(chunk) => chunk.as_message(0, 0),
        }
    }

    unsafe fn from_message(msg: GenericMessage) -> Option<Self> {
        match msg.data {
            MessageType::Scalar(0, id, _, _) => Some(WindowServerMessage::RequestClose { id }),
            MessageType::Scalar(1, id, creation_id, 0) => {
                Some(WindowServerMessage::ConfirmCreation { id, creation_id })
            }
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
    CreateWindow {
        dimensions: Dimension,
        creation_id: u64,
    },
    SubmitRender(MemoryChunk<WindowChunk>),
}

impl MessageData for WindowClientMessage {
    fn into_message(self) -> MessageType {
        match self {
            WindowClientMessage::CreateWindow {
                dimensions,
                creation_id,
            } => MessageType::Scalar(
                0,
                dimensions.width as u64,
                dimensions.height as u64,
                creation_id,
            ),
            WindowClientMessage::SubmitRender(chunk) => chunk.as_message(0, 0),
        }
    }

    unsafe fn from_message(msg: GenericMessage) -> Option<Self> {
        match msg.data {
            MessageType::Scalar(0, width, height, creation_id) => {
                Some(WindowClientMessage::CreateWindow {
                    dimensions: Dimension::new(width as u32, height as u32),
                    creation_id,
                })
            }
            MessageType::Chunk { data: (0, _), .. } => {
                let chunk = msg.data.as_chunk::<WindowChunk>();
                chunk.map(|chunk| WindowClientMessage::SubmitRender(chunk))
            }
            _ => None,
        }
    }
}
