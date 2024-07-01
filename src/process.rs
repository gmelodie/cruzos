use crate::{gdt, interrupts::INTERRUPT_CONTEXT_SIZE, prelude::*};

use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::{instructions::interrupts, VirtAddr};

// use alloc::boxed::Box;
use lazy_static::lazy_static;

lazy_static! {
    static ref RUNNING_QUEUE: Locked<ConcurrentDeque<Process>> =
        Locked::new(ConcurrentDeque::new(|| Process::new(|| {}, false)));
    static ref CURRENT_PROC: Locked<Option<Process>> = Locked::new(None);
}

const KERNEL_STACK_SIZE: usize = 4096 * 2;
const USER_STACK_SIZE: usize = 4096 * 5;

static CUR_PID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy)]
pub struct Process {
    id: usize,
    kernel: bool,
    context_addr: u64,

    kernel_stack: [u8; KERNEL_STACK_SIZE],
    kernel_stack_end: u64,
    user_stack: [u8; USER_STACK_SIZE],
    user_stack_end: u64,
}

impl Process {
    pub fn new(func: fn() -> (), kernel: bool) -> Self {
        let id = CUR_PID.fetch_add(1, Ordering::SeqCst);

        let kernel_stack = [0; KERNEL_STACK_SIZE];
        let kernel_stack_end =
            (VirtAddr::from_ptr(kernel_stack.as_ptr()) + KERNEL_STACK_SIZE as u64).as_u64();

        let user_stack = [0; USER_STACK_SIZE];
        let user_stack_end =
            (VirtAddr::from_ptr(user_stack.as_ptr()) + USER_STACK_SIZE as u64).as_u64();

        // init context with rip, rsp, etc.
        let rip = func as usize;
        let rsp = user_stack_end as usize;
        Context::init(context_addr(kernel_stack_end), rip, rsp);

        Process {
            id,
            kernel,
            context_addr: context_addr(kernel_stack_end),

            kernel_stack,
            kernel_stack_end,

            user_stack,
            user_stack_end,
        }
    }
}

fn context_addr(kernel_stack_end: u64) -> u64 {
    kernel_stack_end - INTERRUPT_CONTEXT_SIZE as u64
}

unsafe fn context_from_addr(addr: u64) -> &'static mut Context {
    &mut *(addr as *mut Context)
}

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct Context {
    // These are pushed in the handler function
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,

    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,

    pub r8: usize,
    pub rbp: usize,
    pub rsi: usize,
    pub rdi: usize,

    pub rdx: usize,
    pub rcx: usize,
    pub rbx: usize,
    pub rax: usize,
    // Below is the exception stack frame pushed by the CPU on interrupt
    // Note: For some interrupts (e.g. Page fault), an error code is pushed here
    rip: usize,    // Instruction pointer
    cs: usize,     // Code segment
    rflags: usize, // Processor flags
    rsp: usize,    // Stack pointer
    ss: usize,     // Stack segment
                   // Here the CPU may push values to align the stack on a 16-byte boundary (for SSE)
}

impl Context {
    pub fn init(context_addr: u64, rip: usize, rsp: usize) {
        let context = unsafe { context_from_addr(context_addr) };
        let (cs, ss) = match gdt::get_kernel_segments() {
            (code_selector, data_selector) => (code_selector.0 as usize, data_selector.0 as usize),
        };

        context.rip = rip;
        context.rsp = rsp;
        context.rflags = 0x200; // interrupts enabled
        context.cs = cs;
        context.ss = ss;
    }
}

pub fn new_kernel_process(f: fn() -> ()) {
    let proc = Process::new(f, true);
    interrupts::without_interrupts(|| {
        RUNNING_QUEUE.lock().push(proc);
    });
}

pub fn new_user_process() {} // TODO

// TODO: redo this
pub fn schedule_next(cur_context_addr: usize) -> usize {
    let cur_context: &mut Context = unsafe { context_from_addr(cur_context_addr as u64) };

    let mut running_queue = RUNNING_QUEUE.lock();
    let mut current_thread = CURRENT_PROC.lock();

    if let Some(mut thread) = current_thread.take() {
        // Save the location of the Context struct
        thread.context_addr = cur_context_addr as u64;
        // Put to the back of the queue
        running_queue.push(thread);
    }
    // Get the next thread in the queue
    *current_thread = running_queue.pop();
    match current_thread.as_ref() {
        Some(thread) => {
            // Set the kernel stack for the next interrupt
            gdt::set_interrupt_stack_table(
                gdt::TIMER_INTERRUPT_INDEX as usize,
                VirtAddr::new(thread.kernel_stack_end),
            );
            // Point the stack to the new context
            thread.context_addr as usize
        }
        None => 0, // Timer handler won't modify stack
    }
}
