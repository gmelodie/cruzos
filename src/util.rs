use core::fmt::Write;

use crate::{println, print};


// pub type Result<'a, T> = result::Result<T, &'a str>;

// TODO: error trait

pub struct TestDescAndFn<'test_life> {
    pub name: &'test_life str,
    pub func: &'test_life fn() -> bool,
}


pub fn run_tests(tests: &[&TestDescAndFn]) {
    for t in tests {
        print!("{}...", t.name);
        if !(t.func)() {
            println!("[failed]");
        } else {
            println!("[ok]");
        }
    }

    // exit_qemu(QemuExitCode::Success); // TODO: uncomment when serial module is ready and output
                                         // can be redirected to host
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

