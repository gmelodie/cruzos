
use core::ptr::addr_of;

use x86_64::{structures::{
    gdt::{GlobalDescriptorTable, Descriptor},
    tss::TaskStateSegment,
}, VirtAddr};
use x86_64::registers::segmentation::SegmentSelector;

use lazy_static::lazy_static;

use crate::prelude::*;


pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe {addr_of!(STACK)});
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };
        tss
    };
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        (gdt, Selectors {code_selector, tss_selector})
    };
}

pub struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init_gdt() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    logf!(Level::Info, "Setting up GDT...");

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }

    log!(Level::Info, "OK");
}
