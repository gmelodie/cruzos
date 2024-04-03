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

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER_ADDRESS: usize = 0xb8000;

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
impl core::fmt::Write for VGA {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let byte_vec: &[u8] = s.as_bytes();

        // characters in byte_vec cannot be out of bounds (0 <= c <= 255)
        for &byte in byte_vec.iter() {
            self.writer.write(byte, self.color());
        }

        // TODO: self.writer.flush();

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Position {
    row: usize,
    col: usize,
}

impl Position {
    fn new(row: usize, col: usize) -> Position {
        Position {
            row,
            col,
        }
    }
    /// Converts the current position (row, col) to 0xb8000 (the start of the VGA buffer) plus an offset
    fn to_byte(&mut self) -> *mut u8 {
        (((self.row*BUFFER_WIDTH + self.col) * 2) + VGA_BUFFER_ADDRESS) as *mut u8
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Letter {
    pos: Position,
}

#[repr(C)]
struct Writer {
    cur_pos: Position,
    letters: [Letter; BUFFER_WIDTH*BUFFER_HEIGHT], // height = letters.len() / width
}

impl Writer {

    fn new() -> Writer {
        Writer {
            cur_pos: Position::new(0, 0),
            letters: [Letter{pos: Position::new(0, 0)}; BUFFER_WIDTH*BUFFER_HEIGHT], // TODO: use letters
        }
    }

    fn height(&self) -> usize {
        self.letters.len() / BUFFER_WIDTH
    }

    fn write(&mut self, byte: u8, color: u8) {
        // if row == 25 we are at row number 26, if height is 25 we need to push everything one line up
        if self.cur_pos.row >= self.height() {
            self.cur_pos.row = self.height() - 1; // set row to last row
            // TODO: push all lines back (needs vga buffer)
        }
        if byte == b'\n' {
            self.cur_pos.row += 1;
            self.cur_pos.col = 0;
            return;
        }
        self.cur_pos.col += 1;
        self.print_char(byte, color, self.cur_pos.row, self.cur_pos.col);
    }

    fn print_char(&self, byte: u8, color: u8, row: usize, col: usize) {
        let mut pos = Position{row, col};
        unsafe {
            *pos.to_byte() = byte;
            *pos.to_byte().offset(1 as isize) = color;
        }
    }

}

