use super::*;
use monos_gfx::{input::KeyboardInput, Framebuffer, Input};

struct Window<T> {
    id: u64,
    title: String,
    creation_id: u64,
    on_render: Box<dyn Fn(&mut T, &mut Framebuffer, Input)>,
}

pub struct WindowClient<T> {
    channel: ChannelHandle,
    windows: Vec<Window<T>>,
    next_creation_id: u64,
    app_data: T,
}

#[derive(Debug)]
pub enum WindowClientError {
    ConnectionError,
}

impl<T> WindowClient<T> {
    pub fn new(port: &str, app_data: T) -> Result<Self, WindowClientError> {
        let channel = syscall::connect(port).ok_or(WindowClientError::ConnectionError)?;

        Ok(WindowClient {
            channel,
            windows: Vec::new(),
            next_creation_id: 0,
            app_data,
        })
    }

    fn receive_msg(&self) -> Option<WindowServerMessage> {
        // safety: we know that only WindowServerMessages get sent over this channel
        unsafe { self.channel.receive::<WindowServerMessage>() }
    }

    pub fn create_window<R>(&mut self, title: &str, dimensions: Dimension, on_render: R)
    where
        R: Fn(&mut T, &mut Framebuffer, Input) + 'static,
    {
        let creation_id = self.next_creation_id;
        self.next_creation_id += 1;

        self.windows.push(Window {
            id: 0,
            title: title.to_string(),
            creation_id,
            on_render: Box::new(on_render),
        });

        self.channel.send(WindowClientMessage::CreateWindow {
            dimensions,
            creation_id,
        });
    }

    pub fn update(&mut self) {
        match self.receive_msg() {
            Some(WindowServerMessage::ConfirmCreation { id, creation_id }) => {
                let window = self
                    .windows
                    .iter_mut()
                    .find(|w| w.creation_id == creation_id)
                    .unwrap();
                window.id = id;
            }

            Some(WindowServerMessage::RequestRender(mut chunk)) => {
                let window = self.windows.iter().find(|w| w.id == chunk.id).unwrap();

                chunk.set_title(&window.title);

                let input = Input {
                    keyboard: KeyboardInput {
                        keys: chunk.keys().to_vec(),
                    },
                    mouse: chunk.mouse.clone(),
                };

                let mut fb = chunk.fb();
                (window.on_render)(&mut self.app_data, &mut fb, input);

                self.channel.send(WindowClientMessage::SubmitRender(chunk));
            }

            Some(WindowServerMessage::RequestClose { id }) => {
                self.windows.retain(|w| w.id != id);
            }

            None => {}
        }
    }

    pub fn data(&self) -> &T {
        &self.app_data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.app_data
    }
}
