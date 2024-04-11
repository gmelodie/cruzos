#[allow(unused)]
use crate::{
    prelude::*,
    exit_qemu, QemuExitCode,
    gdt::DOUBLE_FAULT_IST_INDEX,
};

use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pic8259::ChainedPics;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum PICInterrupt {
    Timer = PIC_1_OFFSET,
}

const PIC_1_OFFSET: u8 = 32;

lazy_static! {
    static ref PICS: Mutex<ChainedPics> = Mutex::new(unsafe {ChainedPics::new_contiguous(PIC_1_OFFSET)}); // this is the same as new(PIC_1_OFFSET, PIC_1_OFFSET + 8);
}


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        let double_fault_options = idt.double_fault.set_handler_fn(double_fault);
        unsafe {
            double_fault_options.set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt[PICInterrupt::Timer as u8].set_handler_fn(timer_interrupt);
        // TODO: set handler functions to PIC interrupts
        idt
    };
}

extern "x86-interrupt" fn timer_interrupt(stack_frame: InterruptStackFrame) {
    unsafe {PICS.lock().notify_end_of_interrupt(PICInterrupt::Timer as u8)};
}

extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    log!(Level::Warning, "Got Breakpoint interrupt: {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("Got Double Fault interrupt (error code {}): {:#?}", error_code, stack_frame);
}

pub fn init_idt() {
    logf!(Level::Info, "Setting up IDT...");

    unsafe {PICS.lock().initialize()};
    x86_64::instructions::interrupts::enable();

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
