#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use core::fmt::Write;
use cruzos::{init, println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}


#[test_case]
fn test_println() {
    println!("test_println output");
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

