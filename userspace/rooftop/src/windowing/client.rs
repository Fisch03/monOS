use super::*;
use core::sync::atomic::Ordering;
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
    title: String,
    chunk: Option<MemoryMappedChunk<WindowChunk>>,
    creation_id: u64,
    on_render: Box<dyn Fn(&mut Window, &mut T, Input)>,
}

#[derive(Debug, Clone)]
pub enum QueuedMessage {
    RequestRender,
}

impl QueuedMessage {
    fn into_message(&self, id: u64) -> WindowClientMessage {
        match self {
            QueuedMessage::RequestRender => WindowClientMessage::RequestRender(id),
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
            title: title.to_string(),
            chunk: None,
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
            Some(WindowServerMessage::ConfirmCreation {
                creation_id,
                mut chunk,
            }) => {
                let window = self
                    .windows
                    .iter_mut()
                    .find(|w| w.creation_id == creation_id)
                    .unwrap();
                let id = chunk.id;

                chunk.set_title(&window.title);

                (window.on_render)(
                    &mut Window {
                        id,
                        fb: chunk.fb(),
                        change_update_frequency: None,
                    },
                    &mut self.app_data,
                    Input::default(),
                );

                window.chunk = Some(chunk);

                self.message_queue.retain(|(expected_creation_id, msg)| {
                    if *expected_creation_id == creation_id {
                        self.channel.send(msg.into_message(id));
                        false
                    } else {
                        true
                    }
                });
            }

            Some(WindowServerMessage::RequestClose { id }) => self.windows.retain(|w| {
                if let Some(chunk) = &w.chunk {
                    chunk.id != id
                } else {
                    true
                }
            }),

            None => {}
        }

        self.windows
            .iter_mut()
            .filter(|w| {
                if let Some(chunk) = &w.chunk {
                    chunk.needs_render.load(Ordering::Relaxed)
                } else {
                    false
                }
            })
            .for_each(|window| {
                let chunk = window.chunk.as_mut().unwrap();

                let input = if chunk.focused {
                    Input {
                        keyboard: KeyboardInput {
                            keys: chunk.keys().to_vec(),
                        },
                        mouse: chunk.mouse.clone(),
                    }
                } else {
                    Input::default()
                };

                let mut window_data = Window {
                    id: chunk.id,
                    fb: chunk.fb(),
                    change_update_frequency: None,
                };

                (window.on_render)(&mut window_data, &mut self.app_data, input);

                if let Some(frequency) = window_data.change_update_frequency {
                    chunk.update_frequency = frequency;
                }

                chunk.needs_render.store(false, Ordering::Relaxed);
            });
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
                let window = self
                    .windows
                    .iter()
                    .find(|w| w.creation_id == creation_id)
                    .unwrap();

                if let Some(chunk) = &window.chunk {
                    self.channel.send(msg.into_message(chunk.id));
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
}
