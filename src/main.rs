#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use alloc::string::ToString;
use bootloader::{entry_point, BootInfo};

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;

use cruzos::{
    apps::gash::Gash, process, task::simple_executor::SimpleExecutor, task::Task, userspace,
};

use core::arch::asm;
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

fn f1() {
    loop {
        println!("1");
        x86_64::instructions::hlt();
    }
}
fn f2() {
    loop {
        println!("2");
        x86_64::instructions::hlt();
    }
}

/// Main for when tests are not run
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    cruzos::init(boot_info);

    set_logging_level(Level::Debug);

    let l4_table = unsafe { cruzos::memory::active_layer_4_page_table() };
    for entry in l4_table.iter() {
        if !entry.is_unused() {
            // only print used entries
            log!(Level::Info, "{:?}", entry);
        }
    }

    // check if registers are being overwritten on timer interrupt
    // unsafe {
    //     asm!("mov r15, 0x42");
    //     asm!("hlt");
    // }

    // let r15: i64;

    // unsafe {
    //     asm!("nop", lateout("r15") r15);
    // }
    // println!("r15 After: {r15}");

    // show off memory allocation
    let _b = Box::new(56);

    #[cfg(test)]
    test_main();
    log!(Level::Info, "\nCruzOS Running!");

    process::new_kernel_process(f1);
    process::new_kernel_process(f2);

    // let shell = Arc::new(Mutex::new(Gash::new()));

    // // show off async capabilities
    // let mut executor = SimpleExecutor::new(50);
    // // let future1 = example_task(42);
    // // let future2 = example_task(43);
    // executor.spawn(Task::new("Gash", async move {
    //     shell.clone().lock().run().await;
    // }));
    // // executor.spawn(Task::new(future1));
    // executor.run();

    cruzos::hlt_loop()
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
