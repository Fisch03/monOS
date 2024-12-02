use super::*;
use crate::messaging::*;

pub fn serve(port: &str) -> Option<PartialReceiveChannelHandle> {
    let ptr = port.as_ptr() as u64;
    let len = port.len() as u64;

    let handle: Option<PartialReceiveChannelHandle> = None;

    let handle_ptr = &handle as *const _;
    unsafe {
        syscall_4(
            Syscall::new(SyscallType::Serve),
            ptr,
            len,
            handle_ptr as u64,
            ChannelLimit::Unlimited.into(),
        )
    };

    handle
}

pub fn receive_any() -> Option<GenericMessage> {
    let mut message: Option<GenericMessage> = None;

    let message_ptr = &mut message as *mut _;
    unsafe { syscall_1(Syscall::new(SyscallType::ReceiveAny), message_ptr as u64) };

    message
}

pub fn receive(handle: ChannelHandle) -> Option<GenericMessage> {
    let mut message: Option<GenericMessage> = None;

    let message_ptr = &mut message as *mut _;
    unsafe {
        syscall_1(
            Syscall::new(SyscallType::Receive).with_handle(handle),
            message_ptr as u64,
        )
    };

    message
}

pub unsafe fn receive_as<T: MessageData>(handle: ChannelHandle) -> Option<T> {
    receive(handle).and_then(|msg| T::from_message(msg))
}

pub fn send<T: MessageData>(handle: ChannelHandle, data: T) {
    let mut flags = SyscallFlags::default();

    let (a, b, c, d) = match data.into_message() {
        MessageType::Scalar(a, b, c, d) => (a, b, c, d),
        MessageType::Chunk {
            address,
            size,
            data,
            is_mmapped,
        } => {
            flags.set_is_chunk();

            if is_mmapped {
                flags.set_is_mmapped();
            }

            (address, size, data.0, data.1)
        }
    };

    unsafe {
        syscall_4(
            Syscall::new(SyscallType::Send)
                .with_handle(handle)
                .with_flags(flags),
            a,
            b,
            c,
            d,
        )
    };
}

pub fn connect(port: &str) -> Option<ChannelHandle> {
    let port_ptr = port.as_ptr() as u64;
    let port_len = port.len() as u64;

    let mut handle: Option<ChannelHandle> = None;

    // SAFETY: the parameters come from a valid string slice and the handle we just created
    unsafe {
        syscall_3(
            Syscall::new(SyscallType::Connect),
            port_ptr,
            port_len,
            &mut handle as *mut _ as u64,
        )
    };

    handle
}

pub fn request_chunk<T: Sized + 'static>() -> Option<MemoryChunk<T>> {
    let address = unsafe {
        syscall_1(
            Syscall::new(SyscallType::RequestChunk),
            core::mem::size_of::<T>() as u64,
        )
    };

    match address {
        0 => None,
        _ => Some(unsafe { MemoryChunk::new(address as *mut T) }),
    }
}
