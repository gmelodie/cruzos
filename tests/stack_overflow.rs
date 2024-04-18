#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

#[allow(unused)]
use cruzos::{prelude::*, should_panic};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(stack_overflow_main);

pub fn stack_overflow_main(_boot_info: &'static BootInfo) -> ! {
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
#[test_case]
fn test_handle_kernel_stack_overflow() {
    should_panic();
    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow();
    }

    // trigger stack overflow (should switch to interrupt stack)
    stack_overflow();
    // fails if this panics instead of successfully recovering
}



