#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};

extern crate alloc;

use alloc::boxed::Box;

use core::panic::PanicInfo;

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
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    cruzos::init(boot_info);

    let l4_table = unsafe { cruzos::memory::active_layer_4_page_table() };
    for entry in l4_table.iter() {
        if !entry.is_unused() {
            // only print used entries
            log!(Level::Info, "{:?}", entry);
        }
    }

    let b = Box::new(56);

    #[cfg(test)]
    test_main();
    log!(Level::Info, "\nCruzOS Running!");

    cruzos::hlt_loop()
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
