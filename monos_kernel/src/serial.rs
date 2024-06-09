use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

#[allow(dead_code)]
pub static SERIAL1: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    Mutex::new(serial_port)
});

#[macro_export]
macro_rules! dbg {
    ($val:expr) => {{
        use core::fmt::Write;
        let val = $val;
        $crate::serial::SERIAL1
            .lock()
            .write_fmt(format_args!("{} = {:#?}\n\n", stringify!($val), val))
            .unwrap();

        val
    }};
}

#[macro_export]
macro_rules! dbg_compact {
    ($val:expr) => {{
        use core::fmt::Write;
        let val = $val;
        $crate::serial::SERIAL1
            .lock()
            .write_fmt(format_args!("{} = {:?}\n\n", stringify!($val), val))
            .unwrap();

        val
    }};
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use crate::interrupts::without_interrupts;
        use core::fmt::Write;

        without_interrupts(|| {
            $crate::serial::SERIAL1.lock().write_fmt(format_args!($($arg)*)).unwrap();
        });
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        $crate::print!($($arg)*);
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}
