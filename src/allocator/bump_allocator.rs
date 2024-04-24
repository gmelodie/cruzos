use alloc::alloc::{GlobalAlloc, Layout};

use crate::{
    allocator::{align_up, HEAP_END, HEAP_START},
    log,
    prelude::*,
    util::Locked,
};

pub struct BumpAllocator {
    alloc_refs: usize,
    alloc_start: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            alloc_refs: 0,
            alloc_start: HEAP_START,
        }
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        log!(Level::Debug, "allocating {} bytes", layout.size());

        // align memory
        let alloc_addr = align_up(self.lock().alloc_start, &layout);

        // check if we can allocate layout.size()
        let new_alloc_end = alloc_addr + layout.size();
        if new_alloc_end > HEAP_END {
            panic!("Not enough space on heap. New alloc end is past actual heap end.");
        }

        // check if there's enough space
        let new_heap_end = alloc_addr + layout.size();
        if new_heap_end > HEAP_END {
            panic!("Not enough space left to allocate");
        }

        // alloc space
        self.lock().alloc_refs += 1;
        self.lock().alloc_start = alloc_addr + layout.size();

        // no need to map since init() already maps it all

        alloc_addr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, layout: Layout) {
        log!(Level::Debug, "deallocating {} bytes", layout.size());
        // decrement alloc_refs
        self.lock().alloc_refs -= 1;

        if self.lock().alloc_refs == 0 {
            self.lock().alloc_refs = 0;
            self.lock().alloc_start = HEAP_START;
        }
    }
}
