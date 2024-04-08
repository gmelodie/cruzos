#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

// use core::fmt::Write;
// use cruzos::serial_println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}


// #[test_case]
// fn test_breakpoint() {
//     x86_64::instructions::interrupts::int3(); // call breakpoint
//     // fails if this panics instead of successfully returning to execution
// }


