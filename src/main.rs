#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga;
mod util;
use util::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &str = "Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {

    vga::print(HELLO, vga::Color::White).unwrap();

    loop {}
}
