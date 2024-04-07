use core::fmt::Write;

use crate::{serial_println, serial_print};


// pub type Result<'a, T> = result::Result<T, &'a str>;

// TODO: error trait


pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        serial_print!("{}...", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn run_tests(tests: &[&dyn Testable]) {
    serial_println!("\nTests");
    serial_println!("-----");
    for t in tests {
        t.run();
    }

    exit_qemu(QemuExitCode::Success);
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

