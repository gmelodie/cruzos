use crate::prelude::*;
use core::result;

pub type Result<'a, T> = result::Result<T, String>;

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
