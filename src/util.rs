use core::fmt::Write;

use crate::{println, print};


// pub type Result<'a, T> = result::Result<T, &'a str>;

// TODO: error trait
// TODO: TestDesc struct
// TODO: #[test] proc_macro that creates a TestDesc given a fn item

pub fn run_tests(tests: &[&dyn Fn() -> bool]) {
    for t in tests {
        print!("{}...", stringify!(t));
        if !t() {
            println!("[failed]");
        } else {
            println!("[ok]");
        }
    }
    // TODO: exit QEMU
}
