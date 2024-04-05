#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

mod util;
mod vga;

use vga::stdout;

// use util::Result;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    writeln!(stdout(), "Formated {} string", 12).unwrap();
    for _ in 0..23 {
        // change to 24 to see first line "disappear" aka get scrolled up
        writeln!(stdout(), "another line").unwrap();
    }
    writeln!(stdout(), "last line").unwrap();
    println!("another last line");
    panic!("some panic msg");

    loop {}
}
