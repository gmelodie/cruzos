use alloc::alloc::{GlobalAlloc, Layout};
use bootloader::bootinfo::MemoryMap;
use core::ptr::null_mut;
use x86_64::structures::paging::{page_table::PageTableFlags, Page};
use x86_64::VirtAddr;

use crate::bump_allocator::BumpAllocator;
use crate::memory;
#[allow(unused)]
use crate::prelude::*;
use crate::util::Locked;

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
// static ALLOCATOR: HeapAllocator = HeapAllocator {};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub struct HeapAllocator {}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called")
    }
}

/// Ensures that start_addr is correctly aligned by layout.align().
/// As almost all of rust dynamic types are base 2 aligned, this will rarely be needed.
/// Still good to have.
pub fn align_up(start_addr: usize, layout: &Layout) -> usize {
    let align = layout.align();
    let remainder = start_addr % align;
    let aligned_addr = match remainder {
        0 => start_addr, // address is aligned
        _ => {
            let difference = align - remainder;
            start_addr + difference
        }
    };

    aligned_addr
}

/// Maps all the heap virtual memory locations to usable physical memory frames.
pub fn init<'init_life>(memory_map: &'init_life MemoryMap) {
    let mut frame_allocator = unsafe { memory::FrameAllocator::new(memory_map) };
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end = heap_start + HEAP_SIZE as u64 - 1;
    let heap_end_page = Page::containing_address(heap_end);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        memory::map_virt(page, flags, &mut frame_allocator);
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use alloc::{boxed::Box, string::String};

    #[test_case]
    fn test_alloc_mem() {
        let mut actual = Box::new(56);
        let expected = 57;
        *actual += 1;
        assert_eq!(*actual, expected);
    }

    #[test_case]
    fn test_alloc_many() {
        for i in 0..1000 {
            let mut actual = Box::new(i);
            *actual += 1;
            let expected = i + 1;
            assert_eq!(*actual, expected);
        }
    }

    #[test_case]
    fn test_alloc_unaligned() {
        let mut string = String::new();
        for _ in 0..1000 {
            string.push('a');
        }
    }
}
