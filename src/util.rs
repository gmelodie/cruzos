use crate::prelude::*;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) }
}

// TODO: error trait

use spin::{mutex::Mutex, Mutex, MutexGuard};

pub struct Locked<T> {
    inner: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }
    pub fn lock(&self) -> MutexGuard<T> {
        // log!(Level::Debug, "locking {}", core::any::type_name::<T>());
        self.inner.lock()
    }
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        // log!(Level::Debug, "locking {}", core::any::type_name::<T>());
        self.inner.try_lock()
    }
}

const LOCK_FREE_DEQUE_SIZE: usize = 1024;

/// Blocking, concurrent circular FIFO queue
/// Pushing appends to beginning of queue
/// Popping removes from end of queue
pub struct Deque<T> {
    items: [T; LOCK_FREE_DEQUE_SIZE],
    end: Mutex<usize>,
    start: Mutex<usize>,
}

impl<T: Copy + Default> LockFreeDeque<T> {
    pub fn new() -> Self {
        LockFreeDeque {
            items: [T::default(); LOCK_FREE_DEQUE_SIZE],
            start: Mutex::new(0),
            end: Mutex::new(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        let start = self.start.lock();
        let end = self.end.lock();

        start == end
    }

    pub fn is_full(&self) -> bool {
        let start = self.start.lock();
        let end = self.end.lock();

        if start == 0 && end == LOCK_FREE_DEQUE_SIZE - 1 || (start == self.end + 1) {
            return true;
        }
        false
    }

    pub fn push(&mut self, item: T) -> Result<()> {
        if self.is_full() {
            return err!("Queue is full");
        }

        // put at end position
        self.items[self.end] = item;
        let old_end = self.end;

        // end goes to beginning of queue when it reaches the end
        self.end = (old_end + 1) % LOCK_FREE_DEQUE_SIZE;
        Ok(())
    }

    /// Pops item from end of queue
    pub fn pop_end(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let old_end = self.end;

        // end goes to end of queue when it reaches the beginning (just as self.start)
        self.end = (old_end - 1 + LOCK_FREE_DEQUE_SIZE) % LOCK_FREE_DEQUE_SIZE;

        Some(self.items[old_end])
    }

    /// Pops item from start of the queue
    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let old_start = self.start;

        // start goes to beginning of queue when it reaches the end (just as self.end)
        self.start = (old_start + 1) % LOCK_FREE_DEQUE_SIZE;

        Some(self.items[old_start])
    }
}
