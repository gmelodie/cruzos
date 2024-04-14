#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[allow(unused)]
use cruzos::prelude::*;


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::panic_handler(info)
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    cruzos::init();

    #[cfg(test)]
    test_main(); // tests exit QEMU when done

    cruzos::main();

    cruzos::hlt_loop()
}

