use core::ptr::NonNull;
use volatile::VolatilePtr;

use super::*;
use crate::messaging::*;

pub fn serve(port: &str, limit: ChannelLimit) {
    let ptr = port.as_ptr() as u64;
    let len = port.len() as u64;

    // SAFETY: the parameters come from a valid string slice
    unsafe { syscall_3(Syscall::new(SyscallType::Serve), ptr, len, limit.into()) };
}

pub fn receive_any() -> Option<Message> {
    let mut message: Option<Message> = None;

    let message_ptr = &mut message as *mut _;
    unsafe { syscall_1(Syscall::new(SyscallType::ReceiveAny), message_ptr as u64) };

    let message_ptr = unsafe { VolatilePtr::new(NonNull::new(message_ptr).unwrap()) };
    message_ptr.read()
}

pub fn receive(handle: ChannelHandle) -> Option<Message> {
    let mut message: Option<Message> = None;

    let message_ptr = &mut message as *mut _;
    unsafe {
        syscall_1(
            Syscall::new(SyscallType::Receive).with_handle(handle),
            message_ptr as u64,
        )
    };

    let message_ptr = unsafe { VolatilePtr::new(NonNull::new(message_ptr).unwrap()) };
    message_ptr.read()
}

pub unsafe fn receive_as<T: MessageData>(handle: ChannelHandle) -> Option<T> {
    receive(handle).and_then(|msg| T::from_message(&msg))
}

pub fn send<T: MessageData>(handle: ChannelHandle, data: T) {
    let (a, b, c, d) = data.into_message();
    unsafe {
        syscall_4(
            Syscall::new(SyscallType::Send).with_handle(handle),
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

pub fn print(s: &str) {
    let ptr = s.as_ptr() as u64;
    let len = s.len() as u64;

    // SAFETY: the parameters come from a valid string slice
    unsafe { syscall_2(Syscall::new(SyscallType::Print), ptr, len) };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;

        // TODO: figure out why format!() doesn't work
        let mut s = $crate::prelude::String::new();
        let _ = write!(s, $($arg)*);
        $crate::syscall::print(&s);

    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! dbg {
    () => ($crate::println!());
    ($val:expr) => {{
        let val = $val;
        $crate::print!("{} = {:?}\n", stringify!($val), &val);
        val

    }};
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
