#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

mod util;
mod vga;
// use util::Result;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let fg_color = vga::Color::White;
    let bg_color = vga::Color::Black;
    let mut vga = vga::VGA::new(fg_color, bg_color);

    writeln!(vga, "Formated {} string", 12).unwrap();
    for _ in 0..23 {
        // change to 24 to see first line "disappear" aka get scrolled up
        writeln!(vga, "another line").unwrap();
    }
    writeln!(vga, "last line").unwrap();

    loop {}
}
