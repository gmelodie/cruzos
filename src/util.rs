use crate::prelude::*;
use core::cell::UnsafeCell;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) }
}

// TODO: error trait

use spin::{Mutex, MutexGuard};

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

/// Blocks until both locks are acquired, returns the two acquired locks
pub fn lock_both<'m1_life, 'm2_life, T1, T2>(
    m1: &'m1_life Mutex<T1>,
    m2: &'m2_life Mutex<T2>,
) -> (MutexGuard<'m1_life, T1>, MutexGuard<'m2_life, T2>) {
    loop {
        match (m1.try_lock(), m2.try_lock()) {
            (Some(g1), Some(g2)) => return (g1, g2),
            _ => continue,
        };
    }
}

const CONCURRENT_DEQUE_SIZE: usize = 512;

/// Blocking, concurrent circular FIFO queue
/// Pushing appends to beginning of queue
/// Popping removes from end of queue
pub struct ConcurrentDeque<T> {
    items: UnsafeCell<Vec<T>>,
    end: Mutex<usize>,
    start: Mutex<usize>,
}

impl<T: Clone> ConcurrentDeque<T> {
    pub fn new() -> Self {
        ConcurrentDeque {
            items: UnsafeCell::new(Vec::new()),
            start: Mutex::new(0),
            end: Mutex::new(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        let (start, end) = lock_both(&self.start, &self.end);

        *start == *end
    }

    pub fn is_full(&self) -> bool {
        let (start, end) = lock_both(&self.start, &self.end);

        if *start == 0 && *end == CONCURRENT_DEQUE_SIZE - 1 || (*start == *end + 1) {
            return true;
        }
        false
    }

    pub fn push(&self, item: T) -> Result<()> {
        if self.is_full() {
            return err!("Queue is full");
        }

        let mut end = self.end.lock();

        // put at end position
        unsafe {
            let items = &mut *self.items.get();
        }
        items[*end] = item;
        let old_end = *end;

        // end goes to beginning of queue when it reaches the end
        *end = (old_end + 1) % CONCURRENT_DEQUE_SIZE;
        Ok(())
    }

    /// Pops item from end of queue
    pub fn pop_end(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let mut end = self.end.lock();

        let old_end = *end;

        // end goes to end of queue when it reaches the beginning (just as self.start)
        *end = (old_end - 1 + CONCURRENT_DEQUE_SIZE) % CONCURRENT_DEQUE_SIZE;

        unsafe {
            let items = &*self.items.get();
        }
        Some(items[old_end].clone())
    }

    /// Pops item from start of the queue
    pub fn pop(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let mut start = self.start.lock();
        let old_start = *start;

        // start goes to beginning of queue when it reaches the end (just as self.end)
        *start = (old_start + 1) % CONCURRENT_DEQUE_SIZE;

        unsafe {
            let items = &*self.items.get();
        }
        Some(items[old_start].clone())
    }

    /// Synchronizes two ConcurrentDeque queues
    /// Makes the two queues equal to push[pop.start, push.end]
    pub fn sync(push: &mut ConcurrentDeque<T>, pop: &mut ConcurrentDeque<T>) {
        let (mut push_start, push_end) = lock_both(&push.start, &push.end);
        let (pop_start, mut pop_end) = lock_both(&pop.start, &pop.end);

        *push_start = *pop_start;
        let old_pop_end = *pop_end;
        *pop_end = *push_end;

        for i in old_pop_end..*push_end {
            let idx = (i + CONCURRENT_DEQUE_SIZE) % CONCURRENT_DEQUE_SIZE; // need this in case we circled back to start of list
            unsafe {
                let pop_items = &mut *pop.items.get();
                let push_items = &*push.items.get();
                pop_items[idx] = push_items[idx].clone();
            }
        }
    }
}

unsafe impl<T: Send> Send for ConcurrentDeque<T> {}
unsafe impl<T: Send> Sync for ConcurrentDeque<T> {}
