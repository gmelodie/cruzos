#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use core::fmt::Write;

use cruzos::vga::stdout;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use cruzos::{
        println,
        exit_qemu,
        QemuExitCode,
    };
    println!("{info}");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    writeln!(stdout(), "CRUZOS Running...").unwrap();

    #[cfg(test)]
    test_main();

    loop {}
}

