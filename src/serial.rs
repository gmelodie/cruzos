use uart_16550::SerialPort;
use lazy_static::lazy_static;
use spin::Mutex;

const SERIAL_IO_PORT: u16 = 0x3F8;

lazy_static! {
    pub static ref SERIAL: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL_IO_PORT) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[macro_export]
macro_rules! serial_print {
    ($($tt:tt)*) => (write!($crate::serial::SERIAL.lock(), "{}", format_args!($($tt)*)).unwrap());
}

#[macro_export]
macro_rules! serial_println {
    ($($tt:tt)*) => ($crate::serial_print!("{}\n", format_args!($($tt)*)));
}

