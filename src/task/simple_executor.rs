use crate::prelude::*;

use super::Task;
use alloc::vec::Vec;
use spin::Mutex;

pub struct SimpleExecutor<'executor_life> {
    tasks: Vec<Task<'executor_life>>,
    is_running: Mutex<()>,
}

impl<'executor_life> SimpleExecutor<'executor_life> {
    pub fn new(capacity: usize) -> Self {
        SimpleExecutor {
            tasks: Vec::with_capacity(capacity),
            is_running: Mutex::new(()),
        }
    }

    pub fn spawn(&mut self, task: Task<'executor_life>) {
        self.is_running.lock();
        self.tasks.push(task);
    }

    pub fn run(&mut self) {
        let mut pending_tasks = self.tasks.len();
        while pending_tasks > 0 {
            self.is_running.lock();
            // TODO: use waker aka add a filter to check if wake was called
            // for (i, task) in self.tasks.iter_mut().filter(|t| t.was_waked()).enumerate() {
            for (i, task) in self.tasks.iter_mut().enumerate() {
                log!(Level::Debug, "polling task {i}");
                let _poll_result = task.poll();
            }

            self.tasks.retain(|task| !task.is_ready()); // retain tasks that are not ready
            pending_tasks = self.tasks.len();
        }
    }
}
