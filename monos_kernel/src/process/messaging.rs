use super::{Process, CURRENT_PROCESS, PROCESS_QUEUE};
use alloc::{string::String, vec, vec::Vec};
use core::sync::atomic::{AtomicU32, Ordering};
use crossbeam_queue::ArrayQueue;
pub use monos_std::messaging::{ChannelHandle, Message};
use spin::{Lazy, RwLock};

const MAX_QUEUE_SIZE: usize = 10;

static PORTS: Lazy<RwLock<Vec<Port>>> = Lazy::new(|| {
    let ports = vec![
        Port::new("sys.keyboard", PortType::System(SystemPort::Keyboard)),
        Port::new("sys.mouse", PortType::System(SystemPort::Mouse)),
    ];

    RwLock::new(ports)
});

static NEXT_SYS_HANDLE: AtomicU32 = AtomicU32::new(0);

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

#[derive(Debug)]
enum PortType {
    System(SystemPort),
    Process(usize),
}

#[derive(Debug)]
enum SystemPort {
    Keyboard,
    Mouse,
}

#[derive(Debug)]
pub struct Mailbox {
    queue: ArrayQueue<Message>,
}

impl Mailbox {
    pub fn new() -> Mailbox {
        Mailbox {
            queue: ArrayQueue::new(MAX_QUEUE_SIZE),
        }
    }

    pub fn send(&self, message: Message) {
        if let Err(_message) = self.queue.push(message) {
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
    let from_handle = ChannelHandle::new(
        connecting_process.id(),
        connecting_process.channels.len() as u16 - 1,
    );

    let ports = PORTS.read();
    let port = ports
        .iter()
        .find(|p| p.name == port)
        .ok_or(ConnectError::PortNotFound)?;

    let to_handle = match &port.port_type {
        PortType::System(system_port) => {
            let to_handle = ChannelHandle::new(NEXT_SYS_HANDLE.fetch_add(1, Ordering::Relaxed), 0);

            match system_port {
                SystemPort::Keyboard => crate::dev::keyboard::add_listener(from_handle),
                SystemPort::Mouse => crate::dev::mouse::add_listener(from_handle),
            }
            to_handle
        }
        PortType::Process(_pid) => todo!("connect to process port"),
    };

    crate::println!(
        "connected pid {} -> pid {} on port '{}'",
        from_handle.thread(),
        to_handle.thread(),
        port.name
    );

    Ok(to_handle)
}

pub fn send(message: Message, handle: ChannelHandle) {
    let receiver = handle.thread();

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

    if let Some(mailbox) = process.channels.get(handle.channel() as usize) {
        mailbox.send(message);
    } else {
        todo!("handle process without open channel")
    }
}
