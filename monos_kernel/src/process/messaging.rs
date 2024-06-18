use alloc::{string::String, vec::Vec};
use crossbeam_queue::ArrayQueue;
use monos_std::messaging::Message;
use spin::RwLock;

const MAX_QUEUE_SIZE: usize = 100;

pub static OPEN_PORTS: RwLock<Vec<Port>> = RwLock::new(Vec::new());

pub struct Port {
    name: String,
    port_type: PortType,
}

pub enum PortType {
    System(SystemPort),
    Process(usize),
}

pub enum SystemPort {
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
}
