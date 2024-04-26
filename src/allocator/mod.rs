use alloc::alloc::Layout;
use bootloader::bootinfo::MemoryMap;
use x86_64::structures::paging::{page_table::PageTableFlags, Page};
use x86_64::VirtAddr;

use crate::memory;
#[allow(unused)]
use crate::prelude::*;
use crate::util::Locked;

use crate::allocator::bump_allocator::BumpAllocator;
use crate::allocator::linked_list_allocator::LinkedListAllocator;

mod bump_allocator;
mod linked_list_allocator;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

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
    logf!(Level::Info, "Mapping heap...");
    let mut frame_allocator = unsafe { memory::FrameAllocator::new(memory_map) };
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end = heap_start + HEAP_SIZE as u64 - 1;
    let heap_end_page = Page::containing_address(heap_end);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        memory::map_virt(page, flags, &mut frame_allocator);
    }
    log!(Level::Info, "OK");
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{boxed::Box, string::String, vec::Vec};

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

    #[test_case]
    fn many_boxes_long_lived() {
        let long_lived = Box::new(1); // new
        for _ in 0..HEAP_SIZE {
            let _x: Vec<usize> = Vec::with_capacity(100);
            // let x = Box::new(i);
            // assert_eq!(*x, i);
        }
        assert_eq!(*long_lived, 1); // new
    }
}
