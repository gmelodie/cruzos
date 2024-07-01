use alloc::{boxed::Box, string::ToString, sync::Arc, task::Wake};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    task::{Context, Poll},
};

#[allow(unused)]
use crate::prelude::*;

pub mod simple_executor;

pub static PID: AtomicUsize = AtomicUsize::new(0);

pub struct Task {
    name: String,
    future: Pin<Box<dyn Future<Output = ()>>>,
    id: usize,
    ready: AtomicBool,
    waker: Arc<TaskWaker>,
}

impl Task {
    pub fn new(name: &str, future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            name: name.to_string(),
            future: Box::pin(future),
            id: PID.fetch_add(1, Ordering::SeqCst),
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
