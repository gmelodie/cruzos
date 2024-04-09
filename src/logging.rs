use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref LOG_LEVEL: Mutex<Level> = Mutex::new(Level::Debug);
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
#[repr(usize)]
pub enum Level {
    Error = 0,
    Warning = 1,
    Info = 2,
    Debug = 3
}

/// Prints a log message if level is higher or equal than current log level
/// # Examples
/// ```
/// log!(LEVEL::Debug, "formated {} logs", 12);
/// ```
#[macro_export]
macro_rules! log {
    ($level:path, $($tt:tt)*) => (
        if $level >= get_logging_level() {
            $crate::serial_print!("{}\n", format_args!($($tt)*));
            $crate::print!("{}\n", format_args!($($tt)*));
        }
    );
}


pub fn set_logging_level(level: Level) {
    *LOG_LEVEL.lock() = level;
}

pub fn get_logging_level() -> Level {
    let level = LOG_LEVEL.lock();
    *level
}
