use crate::LOWER_HALF_END;

use monos_std::messaging::*;

use crate::process::messaging::{connect, send};

pub fn sys_connect(arg1: u64, arg2: u64, arg3: u64) {
    assert!(arg1 < LOWER_HALF_END);
    assert!(arg1 + arg2 < LOWER_HALF_END);

    let port = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            arg1 as *const u8,
            arg2 as usize,
        ))
        .expect("invalid utf8 string")
    };

    let handle_ptr = arg3 as *mut Option<ChannelHandle>;
    let handle = unsafe { &mut *handle_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    *handle = connect(port, current_proc.as_mut()).ok();
}

pub fn sys_receive(handle: ChannelHandle, arg1: u64) {
    let message_ptr = arg1 as *mut Option<Message>;
    let message = unsafe { &mut *message_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    *message = current_proc.receive(handle.recv_part());
}

pub fn sys_receive_any(arg1: u64) {
    let message_ptr = arg1 as *mut Option<Message>;
    let message = unsafe { &mut *message_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    *message = current_proc.receive_any();
}

pub fn sys_send(handle: ChannelHandle, arg1: u64, arg2: u64, arg3: u64, arg4: u64) {
    let data = (arg1, arg2, arg3, arg4);

    let message = {
        let current_proc = crate::process::CURRENT_PROCESS.read();
        let current_proc = current_proc.as_ref().unwrap();

        Message {
            sender: PartialSendChannelHandle {
                target_process: current_proc.id(),
                target_channel: handle.own_channel,
            },
            data,
        }
    };

    send(message, handle.send_part());
}
