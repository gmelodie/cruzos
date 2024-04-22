#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cruzos::run_tests)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use core::panic::PanicInfo;

use cruzos::memory;
#[allow(unused)]
use cruzos::prelude::*;

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::test_panic_handler(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cruzos::panic_handler(info)
}

entry_point!(kernel_main);

/// Main for when tests are not run
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    cruzos::init(boot_info);

    let l4_table = unsafe { cruzos::memory::active_layer_4_page_table() };
    for entry in l4_table.iter() {
        if !entry.is_unused() {
            // only print used entries
            log!(Level::Info, "{:?}", entry);
        }
    }

    set_logging_level(Level::Debug);
    for i in 0..1000 {
        let mut frame_allocator = unsafe { memory::Allocator::new(&boot_info.memory_map) };
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0 + 16168097 * i));
        let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0x0 + 61168_097 * i));

        log!(
            Level::Debug,
            "Page indexes: {:?} {:?} {:?} {:?}",
            page.p4_index(),
            page.p3_index(),
            page.p2_index(),
            page.p1_index()
        );
        log!(Level::Debug, "Page start addr: {:?}", page.start_address());

        memory::map_to(page, frame, flags, &mut frame_allocator);
    }
    // // get page, frame, flags, and allocator to create mapping
    // let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0));
    // let frame = PhysFrame::containing_address(PhysAddr::new(0xf8000));
    // let mut frame_allocator = unsafe { memory::Allocator::new(&boot_info.memory_map) };
    // let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    // memory::map_to(page, frame, flags, &mut frame_allocator);

    // // write the string `New!` to the screen through the new mapping
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();
    // writeln!(stdout(), "CRUZOS Running...").unwrap();
    log!(Level::Info, "\nCRUZOS Running!");

    cruzos::hlt_loop()
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
