use alloc::alloc::Layout;
use bootloader::bootinfo::MemoryMap;
use spin::mutex::Mutex;
use x86_64::structures::paging::frame;
use x86_64::structures::paging::{page_table::PageTableFlags, Page};
use x86_64::VirtAddr;

use crate::memory;
#[allow(unused)]
use crate::prelude::*;
use crate::util::Locked;

// use crate::allocator::bump_allocator::BumpAllocator;
use crate::allocator::linked_list_allocator::LinkedListAllocator;

mod bump_allocator;
mod linked_list_allocator;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

pub const USER_CODE_START: usize = 0x_5555_5555_0000;
pub const USER_CODE_MAX_SIZE: usize = 4 * 1024; // 4 KiB for user code segments
pub const USER_CODE_MAX_END: usize = USER_CODE_START + USER_CODE_MAX_SIZE;

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> =
    Locked::new(LinkedListAllocator::new(HEAP_START, HEAP_END));
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

static USER_ALLOCATOR: Locked<LinkedListAllocator> =
    Locked::new(LinkedListAllocator::new(USER_CODE_START, USER_CODE_MAX_END));

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<memory::FrameAllocator> =
        Mutex::new(unsafe { memory::FrameAllocator::new(None) });
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
pub fn init(memory_map: &'static MemoryMap) {
    logf!(Level::Info, "Mapping heap...");

    *FRAME_ALLOCATOR.lock() = unsafe { memory::FrameAllocator::new(Some(memory_map)) };

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end = heap_start + HEAP_SIZE as u64 - 1;
    let heap_end_page = Page::containing_address(heap_end);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        memory::map_virt(page, flags);
    }

    // alloc_user_code();

    log!(Level::Info, "OK");
}

// TODO: pass in user code to be written already
// TODO: use USER_CODE_START as offset to make virtual memory magic (for user process it looks like zero)
/// Maps the user_code virtual memory locations to usable physical memory frames.
/// Obs: Dangerous! No memory randomization
fn alloc_user_code(size: usize) {
    assert!(size <= USER_CODE_MAX_SIZE);

    let flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    let user_code_start = VirtAddr::new(USER_CODE_START as u64);
    let user_code_start_page = Page::containing_address(user_code_start);
    let user_code_end = user_code_start + size as u64 - 1;
    let user_code_end_page = Page::containing_address(user_code_end);

    for page in Page::range_inclusive(user_code_start_page, user_code_end_page) {
        memory::map_virt(page, flags);
    }
}

fn free_user_code() {
    let user_code_start = VirtAddr::new(USER_CODE_START as u64);
    let user_code_start_page = Page::containing_address(user_code_start);
    let user_code_end = user_code_start + USER_CODE_MAX_SIZE as u64 - 1;
    let user_code_end_page = Page::containing_address(user_code_end);

    for page in Page::range_inclusive(user_code_start_page, user_code_end_page) {
        memory::unmap_virt(page);
    }
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
