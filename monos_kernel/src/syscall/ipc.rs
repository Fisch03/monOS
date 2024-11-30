use crate::LOWER_HALF_END;

use monos_std::messaging::*;
use monos_std::syscall::SyscallFlags;

use crate::process::messaging::{connect, send};

pub fn sys_serve(name_ptr: u64, name_len: u64, handle_ptr: u64) {
    assert!(name_ptr < LOWER_HALF_END);
    assert!(name_ptr + name_len < LOWER_HALF_END);
    assert!(handle_ptr < LOWER_HALF_END);

    let port = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            name_ptr as *const u8,
            name_len as usize,
        ))
        .expect("invalid utf8 string")
    };

    let handle_ptr = handle_ptr as *mut Option<PartialReceiveChannelHandle>;
    let handle = unsafe { &mut *handle_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    *handle = Some(current_proc.serve(port));
}

pub fn sys_connect(name_ptr: u64, name_len: u64, handle_ptr: u64) {
    assert!(name_ptr < LOWER_HALF_END);
    assert!(name_ptr + name_len < LOWER_HALF_END);

    let port = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            name_ptr as *const u8,
            name_len as usize,
        ))
        .expect("invalid utf8 string")
    };

    let handle_ptr = handle_ptr as *mut Option<ChannelHandle>;
    let handle = unsafe { &mut *handle_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    let res = connect(port, current_proc.as_mut());
    if let Err(ref err) = res {
        crate::println!("sys_connect: failed: {:?}", err);
    }
    *handle = res.ok();
}

pub fn sys_receive(handle: ChannelHandle, message_ptr: u64) {
    let message_ptr = message_ptr as *mut Option<GenericMessage>;
    let message = unsafe { &mut *message_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    *message = current_proc.receive(handle.recv_part());
}

pub fn sys_receive_any(message_ptr: u64) {
    let message_ptr = message_ptr as *mut Option<GenericMessage>;
    let message = unsafe { &mut *message_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    *message = current_proc.receive_any();
}

pub fn sys_send(
    handle: ChannelHandle,
    flags: SyscallFlags,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
) {
    let data = if flags.is_chunk() {
        MessageType::Chunk {
            address: arg1,
            size: arg2,
            data: (arg3, arg4),
        }
    } else {
        MessageType::Scalar(arg1, arg2, arg3, arg4)
    };

    let message = {
        let current_proc = crate::process::CURRENT_PROCESS.read();
        let current_proc = current_proc.as_ref().unwrap();

        GenericMessage {
            sender: PartialSendChannelHandle {
                target_process: current_proc.id(),
                target_channel: handle.own_channel,
            },
            data,
        }
    };

    send(message, handle.send_part(), SendOptions::from(flags));
}

pub fn sys_request_chunk(size: u64) -> u64 {
    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();
    current_proc
        .request_chunk(size)
        .map(|addr| addr.as_u64())
        .unwrap_or_default()
}
