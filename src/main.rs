#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::run_tests)]
#![reexport_test_harness_main = "test_main"]

use core::fmt::Write;
use core::panic::PanicInfo;

mod util;
mod vga;

use vga::stdout;
use util::*;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

fn main() {
    writeln!(stdout(), "Formated {} string", 12).unwrap();
    writeln!(stdout(), "another line").unwrap();
    writeln!(stdout(), "last line").unwrap();
    println!("another last line");
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
    fn test_tests() -> bool {
        true
    }
}
