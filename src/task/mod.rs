use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub mod simple_executor;

pub struct Task<'task_life> {
    future: Pin<Box<dyn Future<Output = ()>>>,
    cx: Pin<Box<Context<'task_life>>>,
    ready: bool,
}

// TODO: waker struct

impl Task<'_> {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::pin(future),
            cx: Box::pin(Context::from_waker(Waker::noop())),
            ready: false,
        }
    }

    pub fn ready(&mut self) {
        self.ready = true;
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn poll(&mut self) -> Poll<()> {
        let cx = &mut self.cx;
        let poll_result = self.future.as_mut().poll(cx);
        if let Poll::Ready(_) = poll_result {
            self.ready = true;
        }
        poll_result
    }
}
