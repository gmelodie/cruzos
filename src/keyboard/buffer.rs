use crate::prelude::*;
lazy_static! {
    pub static ref KEYBOARD_BUFFER: Mutex<Buffer> = Mutex::new(Buffer::new());
}

const BUFFER_SIZE: usize = 4096; // 4KiB

/// Buffer is a circular array
pub struct Buffer {
    buf: [char; BUFFER_SIZE],
    end: usize,
    start: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buf: ['\0'; BUFFER_SIZE],
            end: 0,
            start: 0,
        }
    }

    pub fn push(&mut self, ascii: char) {
        // put at end position
        self.buf[self.end] = ascii;
        let old_end = self.end;

        // end goes to beginning of buffer when it reaches the end
        self.end = (old_end + 1) % BUFFER_SIZE;
    }

    /// Pops a character from start of the buffer (returns 0 (\0) if is empty)
    pub fn pop(&mut self) -> char {
        if self.is_empty() {
            return '\0';
        }
        let old_start = self.start;

        // start goes to beginning of buffer when it reaches the end (just as self.end)
        self.start = (old_start + 1) % BUFFER_SIZE;

        self.buf[old_start]
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
