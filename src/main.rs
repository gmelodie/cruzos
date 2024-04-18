#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
#[allow(unused)]
use cruzos::prelude::*;



#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::panic_handler(info)
}

entry_point!(kernel_main);

/// Main for when tests are not run
pub fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    cruzos::init();

    #[cfg(test)]
    test_main();
    // writeln!(stdout(), "CRUZOS Running...").unwrap();
    log!(Level::Info, "\nCRUZOS Running!");


    cruzos::hlt_loop()
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
