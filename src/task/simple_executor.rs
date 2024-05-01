use alloc::sync::Arc;
use alloc::vec::Vec;
use core::task::Context;
use core::task::Waker;
use spin::Mutex;

use super::Task;

use crate::prelude::*;

pub struct SimpleExecutor {
    tasks: Vec<Task>,
    is_running: Mutex<()>,
}

impl SimpleExecutor {
    pub fn new(capacity: usize) -> Self {
        SimpleExecutor {
            tasks: Vec::with_capacity(capacity),
            is_running: Mutex::new(()),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.is_running.lock();
        self.tasks.push(task);
    }

    pub fn run(&mut self) {
        let mut pending_tasks = self.tasks.len();
        while pending_tasks > 0 {
            self.is_running.lock();
            // Filter for unblocked tasks and poll them
            // Unblocked tasks were unblocked by the waker
            for (i, task) in self
                .tasks
                .iter_mut()
                .filter(|t| !t.waker.lock().blocked)
                .enumerate()
            {
                log!(Level::Debug, "polling task {i}");
                let waker = Waker::from(Arc::clone(&task.waker));
                let mut cx = Context::from_waker(&waker);
                let _poll_result = task.poll(&mut cx);
            }

            self.tasks.retain(|task| !task.is_ready()); // retain tasks that are not ready
            pending_tasks = self.tasks.len();
        }
    }
}
