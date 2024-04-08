// use core::fmt::Write;
// use crate::{println, serial_println};

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt
    };
}


extern "x86-interrupt" fn breakpoint(_stack_frame: InterruptStackFrame) {
    // serial_println!("{:#?}", stack_frame);
    // do nothing when breakpoint is called (for now)
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
}
