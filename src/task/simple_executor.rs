use super::Task;
use alloc::vec::Vec;

pub struct SimpleExecutor {
    tasks: Vec<Task>,
}

impl SimpleExecutor {
    pub fn new(capacity: usize) -> Self {
        SimpleExecutor {
            tasks: Vec::with_capacity(capacity),
        }
    }
    pub fn spawn(&mut self, task: Task) {
        self.tasks.push(task);
    }
    pub fn run(&mut self) {
        let mut active_tasks = self.tasks.len();
        while active_tasks > 0 {
            for task in self.tasks {
                // TODO: create new context
                // task.poll()
            }
            active_tasks = self.tasks.len();
        }
    }
}
