#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use cruzos::prelude::*;

entry_point!(basic_boot_main);

pub fn basic_boot_main(boot_info: &'static BootInfo) -> ! {
    cruzos::init(boot_info);

    #[cfg(test)]
    test_main();

    cruzos::hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
