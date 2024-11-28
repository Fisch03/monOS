use super::*;
use monos_gfx::{input::KeyboardInput, Framebuffer, Input};

pub struct Window<'fb> {
    id: u64,
    pub fb: Framebuffer<'fb>,
    change_update_frequency: Option<UpdateFrequency>,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowHandle {
    CreationId(u64),
    Id(u64),
}

impl<'fb> Window<'fb> {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn set_update_frequency(&mut self, frequency: UpdateFrequency) {
        self.change_update_frequency = Some(frequency);
    }
}

impl<'fb> core::ops::Deref for Window<'fb> {
    type Target = Framebuffer<'fb>;

    fn deref(&self) -> &Self::Target {
        &self.fb
    }
}

impl<'fb> core::ops::DerefMut for Window<'fb> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fb
    }
}

impl From<Window<'_>> for WindowHandle {
    fn from(window: Window) -> Self {
        WindowHandle::Id(window.id)
    }
}

struct InternalWindow<T> {
    id: u64,
    title: String,
    creation_id: u64,
    on_render: Box<dyn Fn(&mut Window, &mut T, Input)>,
}

#[derive(Debug, Clone)]
pub enum QueuedMessage {
    RequestRender,
    SetUpdateFrequency(UpdateFrequency),
}

impl QueuedMessage {
    fn into_message(&self, id: u64) -> WindowClientMessage {
        match self {
            QueuedMessage::RequestRender => WindowClientMessage::RequestRender(id),
            QueuedMessage::SetUpdateFrequency(frequency) => {
                WindowClientMessage::SetUpdateFrequency {
                    id,
                    frequency: *frequency,
                }
            }
        }
    }
}

pub struct WindowClient<T> {
    channel: ChannelHandle,
    windows: Vec<InternalWindow<T>>,
    next_creation_id: u64,
    app_data: T,

    // messages created using a creation_id that hasn't been confirmed yet
    message_queue: Vec<(u64, QueuedMessage)>,
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
            message_queue: Vec::new(),
        })
    }

    fn receive_msg(&self) -> Option<WindowServerMessage> {
        // safety: we know that only WindowServerMessages get sent over this channel
        unsafe { self.channel.receive::<WindowServerMessage>() }
    }

    pub fn create_window<R>(
        &mut self,
        title: &str,
        dimensions: Dimension,
        on_render: R,
    ) -> WindowHandle
    where
        R: Fn(&mut Window, &mut T, Input) + 'static,
    {
        if dimensions.width * dimensions.height > MAX_DIMENSION as u32 {
            panic!("window dimensions too large");
        }

        let creation_id = self.next_creation_id;
        self.next_creation_id += 1;

        self.windows.push(InternalWindow {
            id: 0,
            title: title.to_string(),
            creation_id,
            on_render: Box::new(on_render),
        });

        self.channel.send(WindowClientMessage::CreateWindow {
            dimensions,
            creation_id,
        });

        WindowHandle::CreationId(creation_id)
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

                self.message_queue.retain(|(expected_creation_id, msg)| {
                    if *expected_creation_id == creation_id {
                        self.channel.send(msg.into_message(id));
                        false
                    } else {
                        true
                    }
                });
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

                let mut window_data = Window {
                    id: window.id,
                    fb: chunk.fb(),
                    change_update_frequency: None,
                };
                (window.on_render)(&mut window_data, &mut self.app_data, input);

                if let Some(frequency) = window_data.change_update_frequency {
                    self.channel.send(WindowClientMessage::SetUpdateFrequency {
                        id: window.id,
                        frequency,
                    });
                }

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

    fn send_or_queue(&mut self, handle: WindowHandle, msg: QueuedMessage) {
        match handle {
            WindowHandle::CreationId(creation_id) => {
                if let Some(window) = self
                    .windows
                    .iter()
                    .find(|w| w.creation_id == creation_id && w.id != 0)
                {
                    self.channel.send(msg.into_message(window.id));
                } else {
                    self.message_queue.push((creation_id, msg));
                }
            }
            WindowHandle::Id(id) => {
                self.channel.send(msg.into_message(id));
            }
        }
    }

    pub fn request_render(&mut self, handle: WindowHandle) {
        self.send_or_queue(handle, QueuedMessage::RequestRender);
    }

    pub fn set_update_frequency(&mut self, handle: WindowHandle, frequency: UpdateFrequency) {
        self.send_or_queue(handle, QueuedMessage::SetUpdateFrequency(frequency));
    }
}
