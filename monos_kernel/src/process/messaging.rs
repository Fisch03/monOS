use super::{Process, CURRENT_PROCESS, PROCESS_QUEUE};
use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::String, vec::Vec};
pub use monos_std::messaging::{
    ChannelHandle, GenericMessage, MessageData, MessageType, PartialReceiveChannelHandle,
    PartialSendChannelHandle,
};
use monos_std::ProcessId;
use spin::{Lazy, RwLock};

const MAX_QUEUE_SIZE: usize = 256;

static PORTS: Lazy<RwLock<Vec<Port>>> = Lazy::new(|| RwLock::new(Vec::new()));
static SYS_CHANNELS: Lazy<RwLock<Vec<Option<Box<SystemPortReceiveFn>>>>> =
    Lazy::new(|| RwLock::new(Vec::new()));

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
type SystemPortReceiveFn = dyn Fn(GenericMessage) + Sync + Send;
enum PortType {
    System(Box<SystemPortRegisterFn>),
    Process(PartialSendChannelHandle),
}

impl core::fmt::Debug for PortType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PortType::System(_) => f.debug_tuple("System").finish(),
            PortType::Process(pid) => f.debug_tuple("Process").field(pid).finish(),
        }
    }
}

//static NEXT_SYS_HANDLE: AtomicU16 = AtomicU16::new(0);
pub const SYS_PORT_NO_RECEIVE: Option<fn(GenericMessage)> = None; // helper for system ports without receive function to avoid type system shenanigans

pub fn add_system_port<F, G>(
    name: &str,
    register_fn: F,
    receive_fn: Option<G>,
) -> PartialSendChannelHandle
where
    F: Fn(PartialSendChannelHandle) -> PartialSendChannelHandle + Sync + Send + 'static,
    G: Fn(GenericMessage) + Sync + Send + 'static,
{
    let mut sys_channels = SYS_CHANNELS.write();
    let handle = PartialSendChannelHandle::new(ProcessId(0), sys_channels.len() as u16);
    let receive_fn = receive_fn.map(|f| Box::new(f) as Box<SystemPortReceiveFn>);
    sys_channels.push(receive_fn);

    PORTS
        .write()
        .push(Port::new(&name, PortType::System(Box::new(register_fn))));

    handle
}

pub fn add_process_port(name: &str, pid: ProcessId, channel_id: u16) {
    PORTS.write().push(Port::new(
        name,
        PortType::Process(PartialSendChannelHandle::new(pid, channel_id)),
    ));

    crate::println!("pid {} chan {} opened port '{}'", pid, channel_id, name);
}

#[derive(Debug)]
pub struct Mailbox {
    queue: VecDeque<GenericMessage>,
}

impl Mailbox {
    pub fn new() -> Mailbox {
        Mailbox {
            queue: VecDeque::new(),
        }
    }

    pub fn send(&mut self, message: GenericMessage) {
        if self.queue.len() >= MAX_QUEUE_SIZE {
            if message.sender.target_process == ProcessId(0) {
                // system channels can't block
                crate::println!("warning: dropping a system message because the queue is full!");
                return;
            }
            todo!("block sender until there is space in the queue")
        }
        self.queue.push_back(message);
    }

    pub fn receive(&mut self) -> Option<GenericMessage> {
        self.queue.pop_front()
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
    let ports = crate::process::messaging::PORTS.read();

    if ports.len() > 3 {
        crate::println!("name ptr: {:?}", ports[3].name.as_str() as *const str);
        assert_eq!(ports[3].name, "desktop.windows");
    }

    connecting_process.channels.push(Mailbox::new());
    let channel_id = connecting_process.channels.len() as u16 - 1;

    if ports.len() > 3 {
        assert_eq!(ports[3].name, "desktop.windows");
    }

    let from_handle = PartialSendChannelHandle::new(connecting_process.id(), channel_id);

    let ports = PORTS.read();
    let port = ports
        .iter()
        .find(|p| p.name == port)
        .ok_or(ConnectError::PortNotFound)?;

    let to_handle = match &port.port_type {
        PortType::System(register_fn) => register_fn(from_handle),
        PortType::Process(handle) => handle.clone(),
    };

    crate::println!(
        "connected pid {} chan {} <-> pid {} chan {} on port '{}'",
        from_handle.target_process,
        from_handle.target_channel,
        to_handle.target_process,
        to_handle.target_channel,
        port.name
    );

    Ok(ChannelHandle::from_parts(
        to_handle,
        PartialReceiveChannelHandle::new(channel_id),
    ))
}

pub fn send(message: GenericMessage, receiver_handle: PartialSendChannelHandle) {
    let receiver = receiver_handle.target_process;

    if receiver == ProcessId(0) {
        let sys_channels = SYS_CHANNELS.read();
        if let Some(Some(receive_fn)) = sys_channels
            .get(receiver_handle.target_channel as usize)
            .as_ref()
        {
            receive_fn(message);
        } else {
            crate::println!(
                "process {} tried to send to system channel no. {} without receive function",
                receiver,
                receiver_handle.target_channel
            )
        }

        return;
    }

    let mut current_process = CURRENT_PROCESS.write();
    let mut process_queue = PROCESS_QUEUE.write();
    let process = if current_process.as_ref().is_some()
        && current_process.as_ref().unwrap().id() == receiver
    {
        current_process.as_mut().unwrap()
    } else if let Some(process) = process_queue.iter_mut().find(|p| p.id == receiver) {
        process.as_mut()
    } else {
        crate::println!("process {} not found", receiver);
        todo!("handle blocked process / process not found")
    };

    if let Some(mailbox) = process
        .channels
        .get_mut(receiver_handle.target_channel as usize)
    {
        // crate::println!(
        //     "sending message from pid {} -> {} on port {}",
        //     message.sender.target_process,
        //     receiver,
        //     receiver_handle.target_channel
        // );

        mailbox.send(message);
    } else {
        todo!("handle process without open channel")
    }
}
