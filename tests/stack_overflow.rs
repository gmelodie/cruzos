#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
#[allow(unused)]
use cruzos::{prelude::*, should_panic};

entry_point!(stack_overflow_main);

pub fn stack_overflow_main(boot_info: &'static BootInfo) -> ! {
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
