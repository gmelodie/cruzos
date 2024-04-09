use core::fmt::Write;

#[allow(unused)]
use crate::{exit_qemu, QemuExitCode, println, serial_println};

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.double_fault.set_handler_fn(double_fault);
        idt
    };
}


extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    println!("Got Breakpoint interrupt: {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("Got Double Fault interrupt (error code {}): {:#?}", error_code, stack_frame);
}

pub fn init_idt() {
    // set handler to breakpoint interrupt
    IDT.load();
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test_case]
    fn test_breakpoint() {
        x86_64::instructions::interrupts::int3(); // call breakpoint
        // fails if this panics instead of successfully returning to execution
    }

    // tests that should fail are integration tests (tests/interrupts.rs)
}
