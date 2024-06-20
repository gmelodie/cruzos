#![no_std]
#![feature(noop_waker)] // TODO: remove this when using actual waker in Task
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(run_tests)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_mut_refs)] // for linked_list_allocator (using mut ref in const function)
#![feature(error_in_core)] // for core::error::Error (used in util::Result)

use core::panic::PanicInfo;

#[allow(unused_imports)]
use bootloader::{entry_point, BootInfo};
use prelude::*;

extern crate alloc;

pub mod allocator;

pub mod apps;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod logging;
pub mod memory;
pub mod prelude;
pub mod serial;
pub mod task;
pub mod userspace;
pub mod util;
pub mod vga;

pub fn init(boot_info: &BootInfo) {
    x86_64::instructions::interrupts::disable();

    set_logging_level(Level::Info);
    interrupts::init_idt();
    gdt::init_gdt();
    memory::init(boot_info);
    allocator::init(&boot_info.memory_map);

    x86_64::instructions::interrupts::enable();
}

/// Panic handler for when not testing (called in src/main.rs)
pub fn panic_handler(info: &PanicInfo) -> ! {
    log!(Level::Debug, "normal panic_handler called");
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
    log!(Level::Debug, "test_panic_handler called");
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
entry_point!(test_kernel_main);

/// Main for tests
pub fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    set_logging_level(Level::Warning);

    #[cfg(test)]
    test_main();

    // should never get here
    unreachable!();
    #[allow(unreachable_code)]
    hlt_loop()
}

lazy_static! {
    pub static ref TEST_SHOULD_PANIC: Mutex<bool> = Mutex::new(false);
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
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
    fn test_tests() {}
}
