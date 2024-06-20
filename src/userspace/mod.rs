use crate::{memory, prelude::*};
use core::arch::asm;

/// Creates page table for user code
/// Sets memory addresses and registers
/// Calls function in user mode
pub fn to_user_mode() {
    // create user page table
    // allocate pages for program (how many?)
    // copy data to allocated pages
    unsafe { user_prog_1() };
}

unsafe fn user_prog_1() {
    asm!(
        "\
        nop
        nop
        nop
    "
    );
}
