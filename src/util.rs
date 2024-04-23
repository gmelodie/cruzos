// use core::fmt::Write;

// use crate::{serial_println, serial_print};

// pub type Result<'a, T> = result::Result<T, &'a str>;

// TODO: error trait

use spin::{Mutex, MutexGuard};

pub struct Locked<T> {
    inner: Mutex<T>,
}

impl Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }
    pub fn lock() -> MutexGuard<_, T> {
        self.inner.lock()
    }
}
