#[allow(unused_imports)]
use super::{syscall_1, syscall_2, syscall_3, syscall_4, Syscall};

#[inline(always)]
pub fn print(s: &str) {
    let ptr = s.as_ptr() as u64;
    let len = s.len() as u64;

    // SAFETY: the parameters come from a valid string slice
    unsafe { syscall_2(Syscall::Print, ptr, len) };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::syscall::print(&$crate::prelude::format!($($arg)*));

    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
