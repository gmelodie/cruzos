#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use core::fmt::Write;


pub mod util;
pub mod vga;
pub mod serial;
pub mod interrupts;

pub fn init() {
    interrupts::init_idt();
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("{info}");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    #[cfg(test)]
    test_main(); // tests exit QEMU when done

    loop {}
}


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


#[cfg(test)]
mod tests {
    // use super::*;

    #[test_case]
    fn test_tests(){
    }
}
