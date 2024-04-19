#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
#[allow(unused)]
use cruzos::{prelude::*, should_panic, test_kernel_main};

entry_point!(double_fault_main);

pub fn double_fault_main(boot_info: &'static BootInfo) -> ! {
    cruzos::init(boot_info);

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
