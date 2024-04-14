#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use prelude::*;

pub mod prelude;
pub mod util;
pub mod logging;
pub mod vga;
pub mod serial;
pub mod interrupts;
pub mod keyboard;
pub mod gdt;

pub fn init() {
    set_logging_level(Level::Info);
    interrupts::init_idt();
    gdt::init_gdt();
}

/// Panic handler for when not testing (called in src/main.rs)
pub fn panic_handler(info: &PanicInfo) -> ! {
    log!(Level::Error, "{info}");
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn should_panic() {
    *TEST_SHOULD_PANIC.lock() = true;
}

/// Panic handler for when testing (called by all unit and integration tests)
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    log!(Level::Debug, "Test panic handler called");
    let should_panic = *TEST_SHOULD_PANIC.lock();
    log!(Level::Debug, "Should panic is {should_panic}");
    if !should_panic {
        log!(Level::Error, "{info}");
        exit_qemu(QemuExitCode::Failed);
    } else {
        // this is for testing purposes
        *TEST_SHOULD_PANIC.lock() = false; // reset to false
        serial_println!("[ok]");
        exit_qemu(QemuExitCode::Success);
    }
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}


#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();

    #[cfg(test)]
    test_main(); // tests exit QEMU when done

    main();

    hlt_loop()
}

/// Main for when tests are not run
pub fn main() {
    // writeln!(stdout(), "CRUZOS Running...").unwrap();
    log!(Level::Info, "\nCRUZOS Running!");
}

lazy_static! {
    pub static ref TEST_SHOULD_PANIC: Mutex<bool> = Mutex::new(false);
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
    NotExit,
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
