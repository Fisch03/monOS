use super::{CURRENT_PROCESS, PROCESS_QUEUE};
use alloc::{string::String, vec, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_queue::ArrayQueue;
pub use monos_std::messaging::{ChannelHandle, Message};
use spin::{Lazy, RwLock};

const MAX_QUEUE_SIZE: usize = 100;

static PORTS: Lazy<RwLock<Vec<Port>>> = Lazy::new(|| {
    let ports = vec![
        Port::new("sys.keyboard", PortType::System(SystemPort::Keyboard)),
        Port::new("sys.mouse", PortType::System(SystemPort::Mouse)),
    ];

    RwLock::new(ports)
});
static OPEN_CHANNELS: Lazy<RwLock<Vec<OpenChannel>>> = Lazy::new(|| RwLock::new(Vec::new()));

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0);

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
struct OpenChannel {
    handle: ChannelHandle,
    pid_a: u64,
    pid_b: u64,
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
}

#[derive(Debug)]
pub enum ConnectError {
    PortNotFound,
}

pub fn connect(port: &str, pid: usize) -> Result<ChannelHandle, ConnectError> {
    let ports = PORTS.read();
    let port = ports
        .iter()
        .find(|p| p.name == port)
        .ok_or(ConnectError::PortNotFound)?;

    let (handle, receiver_pid) = match &port.port_type {
        PortType::System(system_port) => {
            let handle = ChannelHandle::new(NEXT_HANDLE.fetch_add(1, Ordering::Relaxed) as u32);
            match system_port {
                SystemPort::Keyboard => crate::dev::keyboard::add_listener(handle),
                SystemPort::Mouse => crate::dev::mouse::add_listener(handle),
            }
            (handle, 0)
        }
        PortType::Process(pid) => todo!("connect to process port"),
    };

    OPEN_CHANNELS.write().push(OpenChannel {
        handle,
        pid_a: pid as u64,
        pid_b: receiver_pid,
    });

    Ok(handle)
}

pub fn send(message: Message) {
    let open_channels = OPEN_CHANNELS.read();
    let open_channel = open_channels
        .iter()
        .find(|c| c.handle == message.handle)
        .unwrap();
    let receiver = if message.sender == open_channel.pid_a {
        open_channel.pid_b
    } else {
        open_channel.pid_a
    };

    let current_process = CURRENT_PROCESS.read();
    if current_process.as_ref().is_some()
        && current_process.as_ref().unwrap().id() == receiver as usize
    {
        let mailbox = current_process.as_ref().unwrap().mailbox.send(message);
    } else if let Some(process) = PROCESS_QUEUE
        .read()
        .iter()
        .find(|p| p.id == receiver as usize)
    {
        let mailbox = process.mailbox.send(message);
    } else {
        todo!("send message to process not in queue")
    }

    crate::println!("sent message to process {}", receiver);
}
