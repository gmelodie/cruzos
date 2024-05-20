#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};

extern crate alloc;

use alloc::boxed::Box;
use cruzos::apps::crash::Crash;
use cruzos::task::simple_executor::SimpleExecutor;
use cruzos::task::Task;

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

async fn example_task(num: u64) {
    log!(Level::Info, "Async number is {num}");
}

/// Main for when tests are not run
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    cruzos::init(boot_info);

    // set_logging_level(Level::Debug);

    let l4_table = unsafe { cruzos::memory::active_layer_4_page_table() };
    for entry in l4_table.iter() {
        if !entry.is_unused() {
            // only print used entries
            log!(Level::Info, "{:?}", entry);
        }
    }

    // show off memory allocation
    let _b = Box::new(56);

    #[cfg(test)]
    test_main();
    log!(Level::Info, "\nCruzOS Running!");

    let shell = Crash::new();
    shell.run();

    // show off async capabilities
    // let mut executor = SimpleExecutor::new(50);
    // let future1 = example_task(42);
    // let future2 = example_task(43);
    // executor.spawn(Task::new(future2));
    // executor.spawn(Task::new(future1));
    // executor.run();

    cruzos::hlt_loop()
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
