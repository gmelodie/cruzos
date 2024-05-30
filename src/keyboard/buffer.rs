use crate::prelude::*;
lazy_static! {
    pub static ref PUSH_BUFFER: Mutex<PushBuffer> = Mutex::new(PushBuffer::new());
    pub static ref POP_BUFFER: Mutex<PopBuffer> = Mutex::new(PopBuffer::new());
}

use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{stream::Stream, task::AtomicWaker};

pub static POP_WAKER: AtomicWaker = AtomicWaker::new();

const BUFFER_SIZE: usize = 4096; // 4KiB

/// Buffer is a circular queue
pub struct Buffer {
    buf: [char; BUFFER_SIZE],
    end: usize,
    start: usize,
}

pub type PopBuffer = Buffer;
pub type PushBuffer = Buffer;

impl Buffer {
    pub fn new() -> Self {
        Self {
            buf: ['\0'; BUFFER_SIZE],
            end: 0,
            start: 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    pub fn is_full(&self) -> bool {
        if self.start == 0 && self.end == BUFFER_SIZE - 1 || (self.start == self.end + 1) {
            return true;
        }
        false
    }
}

/// Synchronizes two buffers
/// Makes the two buffers equal to push[pop.start, push.end]
pub fn sync(push: &mut PushBuffer, pop: &mut PopBuffer) {
    push.start = pop.start;
    pop.end = push.end;
    pop.buf = push.buf.clone();
}

impl PushBuffer {
    pub fn push(&mut self, ascii: char) {
        if self.is_full() {
            return;
        }
        // put at end position
        self.buf[self.end] = ascii;
        let old_end = self.end;

        POP_WAKER.wake();
        // end goes to beginning of buffer when it reaches the end
        self.end = (old_end + 1) % BUFFER_SIZE;
    }
    /// Pops a character from end of the buffer (returns 0 (\0) if is empty)
    pub fn pop_end(&mut self) -> Option<char> {
        if self.is_empty() {
            return None;
        }
        let old_end = self.end;

        // end goes to end of buffer when it reaches the beginning (just as self.start)
        self.end = (old_end - 1 + BUFFER_SIZE) % BUFFER_SIZE;

        Some(self.buf[old_end])
    }
}

impl PopBuffer {
    // TODO: result + return error when empty
    /// Pops a character from start of the buffer (returns 0 (\0) if is empty)
    fn pop(&mut self) -> Option<char> {
        if self.is_empty() {
            // yields because poll_next (below) returns Poll::Pending from this
            return None;
        }
        let old_start = self.start;

        // start goes to beginning of buffer when it reaches the end (just as self.end)
        self.start = (old_start + 1) % BUFFER_SIZE;

        Some(self.buf[old_start])
    }
}

pub struct PopBufferStream;

impl PopBufferStream {
    pub fn new() -> Self {
        Self {}
    }
}

impl Stream for PopBufferStream {
    type Item = char;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if let Some(c) = POP_BUFFER.lock().pop() {
            return Poll::Ready(Some(c));
        }
        POP_WAKER.register(&cx.waker());
        match POP_BUFFER.lock().pop() {
            Some(c) => {
                POP_WAKER.take();
                Poll::Ready(Some(c))
            }
            None => Poll::Pending,
        }
    }
}
