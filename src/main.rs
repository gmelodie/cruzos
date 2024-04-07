#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::fmt::Write;
use core::panic::PanicInfo;

mod util;
mod vga;
mod serial;

use vga::stdout;
use util::*;


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{info}");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn main() {
    writeln!(stdout(), "CRUZOS Running...").unwrap();
    // serial_println!("this is in the console");
    // panic!("some panic msg");

}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main(); // tests exit QEMU when done

    main();

    loop {}
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test_case]
    fn test_tests(){
    }
}
