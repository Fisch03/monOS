use super::*;

pub fn print(s: &str) {
    let ptr = s.as_ptr() as u64;
    let len = s.len() as u64;

    // SAFETY: the parameters come from a valid string slice
    unsafe { syscall_2(Syscall::new(SyscallType::Print), ptr, len) };
}

/// shorthand for `sys_info(SysInfo::SystemTime)` because it's used often
#[inline(always)]
pub fn get_time() -> u64 {
    sys_info(SysInfo::SystemTime)
}

pub fn sys_info(info: SysInfo) -> u64 {
    // SAFETY: the parameter is a valid SysInfo variant
    unsafe { syscall_1(Syscall::new(SyscallType::SysInfo), info as u64) }
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
