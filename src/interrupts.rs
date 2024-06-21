/// Interrupts should not lock important objects like the screen or the serial output
/// as deadlocks might occur (interrupts are a bitch)
#[allow(unused)]
use crate::{exit_qemu, gdt, hlt_loop, keyboard, prelude::*, process, QemuExitCode};

use core::arch::asm;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PICInterrupt {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

const PIC_1_OFFSET: u8 = 32;

lazy_static! {
    pub static ref PICS: Mutex<ChainedPics> = Mutex::new(unsafe {ChainedPics::new_contiguous(PIC_1_OFFSET)}); // this is the same as new(PIC_1_OFFSET, PIC_1_OFFSET + 8);
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // reserved interrupts
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.page_fault.set_handler_fn(page_fault);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        let double_fault_options = idt.double_fault.set_handler_fn(double_fault);
        unsafe {
            double_fault_options.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // PIC interrupts
        let timer_options = idt[PICInterrupt::Timer as u8].set_handler_fn(timer_handler_naked);

        unsafe {
            timer_options.set_stack_index(gdt::TIMER_INTERRUPT_INDEX);
        }

        idt[PICInterrupt::Keyboard as u8].set_handler_fn(keyboard::keyboard_interrupt);
        // TODO: set handler functions to PIC interrupts
        idt
    };
}

/// 1. Pushes registers to stacks
/// 2. Calles timer_interrupt with C calling convention (register are popped into process::Context)
/// 3. Pops registers modified by timer_interrupt
#[naked]
extern "x86-interrupt" fn timer_handler_naked(_stack_frame: InterruptStackFrame) {
    unsafe {
        asm!(
            // Disable interrupts
            "cli",
            // Push registers
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",

            "push rdi",
            "push rsi",
            "push rbp",
            "push r8",

            "push r9",
            "push r10",
            "push r11",
            "push r12",

            "push r13",
            "push r14",
            "push r15",

            // First argument in rdi with C calling convention
            "mov rdi, rsp",
            // Call the hander function
            "call {handler}",

            // Pop scratch registers
            "pop r15",
            "pop r14",
            "pop r13",

            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",

            "pop r8",
            "pop rbp",
            "pop rsi",
            "pop rdi",

            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            // Enable interrupts
            "sti",
            // Interrupt return
            "iretq",
            // Note: Getting the handler pointer here using `sym` operand, because
            // an `in` operand would clobber a register that we need to save, and we
            // can't have two asm blocks
            handler = sym timer_interrupt,
            options(noreturn)
        );
    }
}

extern "C" fn timer_interrupt(context: &process::Context) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(PICInterrupt::Timer as u8)
    };
}

extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    log!(Level::Debug, "Got Breakpoint interrupt: {:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    log!(
        Level::Error,
        "Got Page Fault interrupt ({:?}: {:#?}",
        error_code,
        stack_frame
    );
    hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!(
        "Got Double Fault interrupt (error code {}): {:#?}",
        error_code, stack_frame
    );
}

pub fn init_idt() {
    logf!(Level::Info, "Setting up IDT...");

    unsafe { PICS.lock().initialize() };
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
