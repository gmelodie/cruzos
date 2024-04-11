#[allow(unused)]
use crate::{
    prelude::*,
    exit_qemu, QemuExitCode,
    gdt::DOUBLE_FAULT_IST_INDEX,
};

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        let double_fault_options = idt.double_fault.set_handler_fn(double_fault);
        unsafe {
            double_fault_options.set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}


extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    log!(Level::Warning, "Got Breakpoint interrupt: {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("Got Double Fault interrupt (error code {}): {:#?}", error_code, stack_frame);
}

pub fn init_idt() {
    logf!(Level::Info, "Setting up IDT...");
    // set handler to breakpoint interrupt
    IDT.load();

    log!(Level::Info, "OK");
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[test_case]
    fn test_breakpoint() {
        x86_64::instructions::interrupts::int3(); // call breakpoint
        // fails if this panics instead of successfully returning to execution
    }

    // tests that should fail are integration tests (tests/interrupts.rs)
}
