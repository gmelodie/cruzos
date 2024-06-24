use core::ptr::addr_of;

use x86_64::registers::segmentation::SegmentSelector;
use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use lazy_static::lazy_static;

use crate::prelude::*;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 0;
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 0;

pub const TIMER_INTERRUPT_INDEX: u16 = 1;

lazy_static! {
    pub static ref TSS: Mutex<TaskStateSegment> = {
        let mut tss = TaskStateSegment::new();


        // stack for Double/Page/General Protection Faults
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };

        // stack for Timer Interrupt (context switching)
        tss.interrupt_stack_table[TIMER_INTERRUPT_INDEX as usize] =
            tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize];

        Mutex::new(tss)
    };
}

/// Returns a reference to the TSS
unsafe fn tss_ref() -> &'static TaskStateSegment {
    let tss_ptr = &*TSS.lock() as *const TaskStateSegment;
    &*tss_ptr
}

/// Changes stack of an entry of the TSS
/// Used for context switching
pub fn set_interrupt_stack_table(index: usize, stack_end: VirtAddr) {
    TSS.lock().interrupt_stack_table[index] = stack_end;
}

pub fn get_kernel_segments() -> (SegmentSelector, SegmentSelector) {
    (GDT.1.code_selector, GDT.1.data_selector)
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // this append order is important, kernel_code_segment must come before tss_segment
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(unsafe {tss_ref()}));
        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
                user_code_selector,
                user_data_selector,
            }
        )
    };
}

pub struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
}

pub fn init_gdt() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    logf!(Level::Info, "Setting up GDT...");

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        // DS::set_reg(GDT.1.data_selector); // unused, setting it does nothing
        load_tss(GDT.1.tss_selector);
    }

    log!(Level::Info, "OK");
}

pub fn get_user_segments() -> (SegmentSelector, SegmentSelector) {
    (GDT.1.user_code_selector, GDT.1.user_data_selector)
}
