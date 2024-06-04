use crate::prelude::*;

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
