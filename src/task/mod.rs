use alloc::{boxed::Box, sync::Arc, task::Wake};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{prelude::*, util::Locked};

pub mod simple_executor;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    ready: bool,
    waker: Arc<Locked<TaskWaker>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::pin(future),
            ready: false,
            waker: TaskWaker::new(false),
        }
    }

    pub fn ready(&mut self) {
        self.ready = true;
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        let poll_result = self.future.as_mut().poll(cx);
        match poll_result {
            Poll::Ready(_) => {
                self.ready = true;
            }
            Poll::Pending => {
                log!(Level::Debug, "locking and blocking");
                // reblock task, to be unblocked by Waker
                self.waker.lock().blocked = true;
                log!(Level::Debug, "locked and blocked");
            }
        }
        poll_result
    }
}

struct TaskWaker {
    blocked: bool,
}

impl TaskWaker {
    fn new(blocked: bool) -> Arc<Locked<Self>> {
        Arc::new(Locked::new(TaskWaker { blocked }))
    }
    fn unblock(&mut self) {
        log!(Level::Debug, "unblock called");
        self.blocked = false;
    }
}

impl Wake for Locked<TaskWaker> {
    fn wake(self: Arc<Self>) {
        log!(Level::Debug, "wake called");
        self.lock().unblock();
        log!(Level::Debug, "unblocked");
    }
}
