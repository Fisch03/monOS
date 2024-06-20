use super::{Process, CURRENT_PROCESS, PROCESS_QUEUE};
use alloc::{boxed::Box, string::String, vec::Vec};
use core::sync::atomic::{AtomicU16, Ordering};
use crossbeam_queue::SegQueue;
pub use monos_std::messaging::{
    ChannelHandle, Message, PartialReceiveChannelHandle, PartialSendChannelHandle,
};
use spin::{Lazy, RwLock};

const MAX_QUEUE_SIZE: usize = 100;

static PORTS: Lazy<RwLock<Vec<Port>>> = Lazy::new(|| RwLock::new(Vec::new()));

#[derive(Debug)]
struct Port {
    name: String,
    port_type: PortType,
}

impl Port {
    pub fn new(name: &str, port_type: PortType) -> Port {
        let name = String::from(name);
        Port { name, port_type }
    }
}

type SystemPortRegisterFn =
    dyn Fn(PartialSendChannelHandle) -> PartialSendChannelHandle + Sync + Send;
enum PortType {
    System(Box<SystemPortRegisterFn>),
    Process(usize),
}

impl core::fmt::Debug for PortType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PortType::System(_) => write!(f, "System"),
            PortType::Process(pid) => f.debug_tuple("Process").field(pid).finish(),
        }
    }
}

static NEXT_SYS_HANDLE: AtomicU16 = AtomicU16::new(0);
pub fn add_system_port<F>(name: &str, register_fn: F) -> PartialSendChannelHandle
where
    F: Fn(PartialSendChannelHandle) -> PartialSendChannelHandle + Sync + Send + 'static,
{
    let handle = PartialSendChannelHandle::new(0, NEXT_SYS_HANDLE.fetch_add(1, Ordering::Relaxed));

    PORTS
        .write()
        .push(Port::new(&name, PortType::System(Box::new(register_fn))));

    handle
}

#[derive(Debug)]
pub struct Mailbox {
    queue: SegQueue<Message>, //TODO: figure out why ArrayQueue doesn't work
}

impl Mailbox {
    pub fn new() -> Mailbox {
        Mailbox {
            queue: SegQueue::new(),
        }
    }

    pub fn send(&self, message: Message) {
        if self.queue.len() < MAX_QUEUE_SIZE {
            self.queue.push(message);
        } else {
            todo!("block sender until there is space in the queue")
        }
    }

    pub fn receive(&self) -> Option<Message> {
        self.queue.pop()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

#[derive(Debug)]
pub enum ConnectError {
    PortNotFound,
}

pub fn connect(
    port: &str,
    connecting_process: &mut Process,
) -> Result<ChannelHandle, ConnectError> {
    connecting_process.channels.push(Mailbox::new());
    let channel_id = connecting_process.channels.len() as u16 - 1;

    let from_handle = PartialSendChannelHandle::new(connecting_process.id(), channel_id);

    let ports = PORTS.read();
    let port = ports
        .iter()
        .find(|p| p.name == port)
        .ok_or(ConnectError::PortNotFound)?;

    let to_handle = match &port.port_type {
        PortType::System(register_fn) => register_fn(from_handle),
        PortType::Process(_pid) => todo!("connect to process port"),
    };

    crate::println!(
        "connected pid {} -> pid {} on port '{}'",
        from_handle.target_thread,
        to_handle.target_channel,
        port.name
    );

    Ok(ChannelHandle::from_parts(
        to_handle,
        PartialReceiveChannelHandle::new(channel_id),
    ))
}

pub fn send(message: Message, receiver_handle: PartialSendChannelHandle) {
    let receiver = receiver_handle.target_thread;

    let current_process = CURRENT_PROCESS.read();
    let process_queue = PROCESS_QUEUE.read();
    let process = if current_process.as_ref().is_some()
        && current_process.as_ref().unwrap().id() == receiver
    {
        current_process.as_ref().unwrap()
    } else if let Some(process) = process_queue.iter().find(|p| p.id == receiver) {
        process.as_ref()
    } else {
        todo!("handle blocked process / process not found")
    };

    if let Some(mailbox) = process
        .channels
        .get(receiver_handle.target_channel as usize)
    {
        mailbox.send(message);
    } else {
        todo!("handle process without open channel")
    }
}
