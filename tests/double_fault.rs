#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

#[allow(unused)]
use cruzos::{test_kernel_main, prelude::*, should_panic};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(double_fault_main);

pub fn double_fault_main(_boot_info: &'static BootInfo) -> ! {
    cruzos::init();

    #[cfg(test)]
    test_main();

    cruzos::hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

// there can't be other tests since this should panic
// TODO: force a double fault (this code now throws a page fault since we have the page fault
// handler)
// #[test_case]
// fn test_double_fault() {
//     should_panic();
//     unsafe {
//         *(0xdeadbeef as *mut u8) = 42;
//     }
//     // successful if double_fault handler is called
// }
