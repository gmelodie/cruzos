use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use core::task::Context;
use core::task::Waker;
use spin::Mutex;
// use x86_64::instructions::interrupts;

use super::Task;

use crate::prelude::*;

pub struct SimpleExecutor {
    tasks: BTreeMap<usize, Task>,
    new_tasks: BTreeMap<usize, Task>,
}

impl SimpleExecutor {
    pub fn new(_capacity: usize) -> Self {
        // TODO: set capacity
        SimpleExecutor {
            tasks: BTreeMap::new(),
            new_tasks: BTreeMap::new(),
        }
    }

    /// Spawns a new task, can be called after executor started running
    pub fn spawn(&mut self, task: Task) {
        self.new_tasks.insert(task.id, task);
    }

    /// Adds new_tasks to tasks and clears new_tasks
    fn update_tasks(&mut self) {
        while let Some((pid, task)) = self.new_tasks.pop_last() {
            self.tasks.insert(pid, task);
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            // Add new_tasks to tasks and clear new_tasks
            self.update_tasks();

            assert!(self.new_tasks.is_empty());

            // Filter for unblocked tasks and poll them
            // Unblocked tasks were unblocked by the waker
            for (i, task) in self
                .tasks
                .values_mut()
                .filter(|t| !t.waker.blocked.load(Ordering::SeqCst))
                .enumerate()
            {
                log!(Level::Debug, "polling task {i}");
                let waker = Waker::from(Arc::clone(&task.waker));
                let mut cx = Context::from_waker(&waker);
                let _poll_result = task.poll(&mut cx);
                log!(Level::Debug, "finished polling task {i}");
            }

            self.tasks
                .retain(|_pid, task| !task.ready.load(Ordering::SeqCst)); // retain tasks that are not ready

            // TODO: hlt CPU if no tasks are pending
            // let pending_tasks = self.tasks.len();
            // interrupts::disable();
            // if pending_tasks == 0 {
            //     interrupts::enable_and_hlt();
            // } else {
            //     interrupts::enable();
            // }
        }
    }
}
