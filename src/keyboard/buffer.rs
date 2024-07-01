use crate::prelude::*;
lazy_static! {
    pub static ref PUSH_BUFFER: Mutex<PushBuffer> = Mutex::new(PushBuffer::new(char::default));
    pub static ref POP_BUFFER: Mutex<PopBuffer> = Mutex::new(PopBuffer::new(char::default));
}
pub static POP_WAKER: AtomicWaker = AtomicWaker::new();

use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{stream::Stream, task::AtomicWaker};

pub type PopBuffer = ConcurrentDeque<char>;
pub type PushBuffer = ConcurrentDeque<char>;

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
        log!(Level::Debug, "registering waker");
        POP_WAKER.register(&cx.waker());
        match POP_BUFFER.lock().pop() {
            Some(c) => {
                log!(Level::Debug, "end registering waker (ready)");
                POP_WAKER.take();
                Poll::Ready(Some(c))
            }
            None => {
                log!(Level::Debug, "end registering waker (pending)");
                Poll::Pending
            }
        }
    }
}
