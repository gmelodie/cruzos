#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;

mod vga;
mod util;
// use util::Result;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &str = "Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {

    let fg_color = vga::Color::White;
    let bg_color = vga::Color::Black;
    let mut vga = vga::VGA::new(fg_color, bg_color);

    writeln!(vga, "Formated {} string", 12);
    vga.print(HELLO).unwrap();
    vga.print("\nthere").unwrap();

    loop {}
}
