use spin::{Mutex, Once};
use uart_16550::SerialPort;

pub static SERIAL1: Once<Mutex<SerialPort>> = Once::new();

pub fn init() {
    SERIAL1.call_once(|| {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    });
}

#[macro_export]
macro_rules! dbg {
    ($val:expr) => {{
        use core::fmt::Write;
        $crate::serial::SERIAL1
            .get()
            .unwrap()
            .lock()
            .write_fmt(format_args!("{} = {:#?}\n\n", stringify!($val), $val))
            .unwrap();

        $val
    }};
}
