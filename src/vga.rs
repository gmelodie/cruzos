use crate::util::*;

/// Foreground (text) colors
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[no_mangle]
pub fn print(text: &str, color: Color) -> Result<()> {
    let vga_buffer = 0xb8000 as *mut u8;
    let color = color as u8;
    let byte_vec: &[u8] = text.as_bytes();

    // characters in byte_vec cannot be out of bounds (0 <= c <= 255)
    for (i, &byte) in byte_vec.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = color;
        }
    }

    Ok(())
}


