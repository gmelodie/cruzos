#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[allow(unused)]
use core::fmt::Write;
#[allow(unused)]
use cruzos::{init, serial_println, should_panic};


#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

// there can't be other tests since this should panic
#[test_case]
fn test_recover_from_kernel_stack_overflow() {
    fn stack_overflow() {
        stack_overflow();
    }

    // trigger stack overflow (should switch to interrupt stack)
    stack_overflow();
    // fails if this panics instead of successfully recovering
}



