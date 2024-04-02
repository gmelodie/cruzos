use crate::util::Result;

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

static BUFFER_ROWS: usize = 25;
static BUFFER_COLS: usize = 80;

#[repr(C)]
pub struct VGA {
    pub fg_color: Color,
    pub bg_color: Color,
    writer: Writer,
}


impl VGA {
    pub fn new(fg_color: Color, bg_color: Color) -> VGA {
        VGA {
            fg_color,
            bg_color,
            writer: Writer::new(),
        }
    }

    fn color(&self) -> u8 {
        ((self.bg_color as u8) << 4) | self.fg_color as u8
    }

    #[no_mangle]
    pub fn print(&mut self, text: &str) -> Result<()> {
        let byte_vec: &[u8] = text.as_bytes();

        // characters in byte_vec cannot be out of bounds (0 <= c <= 255)
        for &byte in byte_vec.iter() {
            self.writer.write(byte, self.color());
        }

        Ok(())
    }
}

#[repr(transparent)]
struct Writer {
    pos: *mut u8,
}

impl Writer {

    fn new() -> Writer {
        Writer {
            pos: 0xb8000 as *mut u8,
        }
    }

    // TODO: dont just write one byte at a time
    fn write(&mut self, byte: u8, color: u8) {
            unsafe {
                *self.pos = byte;
                *self.pos.offset(0 as isize * 2 + 1) = color;
                self.pos = self.pos.offset(0 as isize * 2 + 2);
            }
    }
}


























