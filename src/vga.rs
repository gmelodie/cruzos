//!  # VGA
//!  VGA text mode implementation
//!
//!  ## Examples
//!  ```
//!  let fg_color = vga::Color::White;
//!  let bg_color = vga::Color::Black;
//!  let mut vga = vga::Vga::new(fg_color, bg_color);
//!  writeln!(vga, "hello {}", 12);
//!  ```



use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};

// use crate::util::Result;

lazy_static! {
    pub static ref VGA: Mutex<Vga> = Mutex::new(Vga::new(Color::White, Color::Black));
}

#[macro_export]
macro_rules! print {
    ($($tt:tt)*) => (write!($crate::vga::VGA.lock(), "{}", format_args!($($tt)*)).unwrap());
}

#[macro_export]
macro_rules! println {
    ($($tt:tt)*) => ($crate::print!("{}\n", format_args!($($tt)*)));
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER_ADDRESS: *mut u8 = 0xb8000 as *mut u8;


/// Acquires lock for public instance of VGA
pub fn stdout() -> MutexGuard<'static, Vga> {
    VGA.lock()
}


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
#[repr(C)]
pub struct Vga {
    pub fg_color: Color,
    pub bg_color: Color,
    writer: Writer,
}

impl Vga {
    pub fn new(fg_color: Color, bg_color: Color) -> Vga {
        Vga {
            fg_color,
            bg_color,
            writer: Writer::new(),
        }
    }

    fn color(&self) -> u8 {
        ((self.bg_color as u8) << 4) | self.fg_color as u8
    }

}

impl core::fmt::Write for Vga {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let byte_vec: &[u8] = s.as_bytes();

        // characters in byte_vec cannot be out of bounds (0 <= c <= 255)
        for &byte in byte_vec.iter() {
            self.writer.write(byte, self.color());
        }
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
        Position { row, col }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Letter {
    byte: u8,
    color: u8,
}

impl Letter {
    fn new(byte: u8, color: u8) -> Letter {
        Letter { byte, color }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Buffer {
    letters: [[Letter; BUFFER_WIDTH]; BUFFER_HEIGHT], // height = letters.len() / width
}

#[repr(C)]
struct Writer {
    cur_pos: Position,
    buffer: &'static mut Buffer, // height = letters.len() / width
}

impl Writer {
    fn new() -> Writer {
        Writer {
            cur_pos: Position::new(0, 0),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
        }
    }

    fn height(&self) -> usize {
        self.buffer.letters.len()
    }

    fn width(&self) -> usize {
        self.buffer.letters[0].len()
    }

    fn write(&mut self, byte: u8, color: u8) {
        if byte == b'\n' || self.cur_pos.col >= self.width() {
            self.cur_pos.row += 1;
            self.cur_pos.col = 0;
            return;
        }
        // if row == 25 we are at row number 26, if height is 25 we need to push everything one line up
        if self.cur_pos.row >= self.height() {
            for i in 0..self.height() - 1 {
                self.buffer.letters[i] = self.buffer.letters[i + 1].clone();
            }
            self.cur_pos.row = self.height() - 1; // set row to last row
                                                  // clear last row
            self.buffer.letters[self.height() - 1] = [Letter::new(0, 0); BUFFER_WIDTH];
        }
        self.buffer.letters[self.cur_pos.row][self.cur_pos.col] = Letter::new(byte, color);
        self.cur_pos.col += 1;
    }
}