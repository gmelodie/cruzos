use alloc::{boxed::Box, sync::Arc, task::Wake};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use crate::{prelude::*, util::Locked};

pub mod simple_executor;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    ready: AtomicBool,
    waker: Arc<TaskWaker>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            future: Box::pin(future),
            ready: AtomicBool::new(false),
            waker: TaskWaker::new(false),
        }
    }

    pub fn ready(&mut self) {
        self.ready.store(true, Ordering::Relaxed);
    }

    pub fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        let poll_result = self.future.as_mut().poll(cx);
        match poll_result {
            Poll::Ready(_) => {
                self.ready.store(true, Ordering::Relaxed);
            }
            Poll::Pending => {
                // reblock task, to be unblocked by Waker
                self.waker.blocked.store(true, Ordering::Relaxed);
            }
        }
        poll_result
    }
}

struct TaskWaker {
    blocked: AtomicBool,
}

impl TaskWaker {
    fn new(blocked: bool) -> Arc<Self> {
        Arc::new(TaskWaker {
            blocked: AtomicBool::new(blocked),
        })
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.blocked.store(false, Ordering::Relaxed);
    }
}
