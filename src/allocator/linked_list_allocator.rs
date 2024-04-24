use alloc::alloc::{GlobalAlloc, Layout};

use crate::{
    allocator::{align_up, HEAP_END, HEAP_START},
    log,
    prelude::*,
};

#[derive(Debug)]
struct ReusableSpace {
    /// Next is none when there are no further reusable slots
    next: Option<&'static mut ReusableSpace>,
    size: usize,
}

impl ReusableSpace {
    fn new(start_addr: usize, size: usize) -> Self {
        // TODO: use start_addr to save this to memory
        ReusableSpace { next: None, size }
    }

    /// Use this space. Pops it out of list and returns itself.
    fn pop(
        mut to_pop: Option<&'static mut Self>,
        previous: Option<&'static mut Self>,
        next: Option<&'static mut Self>,
    ) -> Self {
        if to_pop.is_none() {
            panic!("Trying to pop unexistent linked list \"heap\" entry");
        }

        match (previous, next) {
            (Some(p), Some(n)) => {
                p.next = Some(n);
            }
            (Some(p), None) => {
                // to_pop is the last entry
                p.next = None
            }
            (None, None) => {} // to_pop is the root and there are no more entries, just take it
            (None, Some(_n)) => {} // to_pop is the root, just take it
        }
        *(to_pop.take().unwrap()) // this should always contain a Some
    }

    fn is_suitable(&self, layout: &Layout) -> bool {
        // block is suitable if aligned address withing block + size is also within block
        let aligned_start_addr = align_up(self.start_addr, layout);
        let end_reusable_block = self.start_addr + self.size - 1;

        aligned_start_addr + layout.size() <= end_reusable_block
    }

    fn find_suitable_reusable_slot(root: &mut Self, layout: &'static Layout) -> Result<usize> {
        let mut prev = None;
        let mut cur = Some(root);

        while cur.is_some() && !cur.unwrap().is_suitable(layout) {
            prev = cur;
            cur = cur.unwrap().next;
        }

        if cur.is_none() {
            return Err("cur does not exist.");
        }

        let space = ReusableSpace::pop(cur, prev, cur.unwrap().next);
        let start_addr = space.start_addr;
        Ok(start_addr)
    }
}

pub struct LinkedListAllocator {
    alloc_refs: usize,
    alloc_start: usize,
    root_reusable: Option<&'static mut ReusableSpace>,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            alloc_refs: 0,
            alloc_start: HEAP_START,
            root_reusable: None,
        }
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        log!(Level::Debug, "allocating {} bytes", layout.size());

        // TODO: find suitable spot in freed memory list
        // where memory is _also_ correctly aligned
        if let Some(root) = self.lock().root_reusable {
            if let Ok(start_addr) = ReusableSpace::find_suitable_reusable_slot(root, &layout) {
                self.lock().alloc_refs += 1;
                return start_addr as *mut u8;
            }
        }

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

        // TODO: put in reusable space

        if self.lock().alloc_refs == 0 {
            self.lock().alloc_refs = 0;
            self.lock().alloc_start = HEAP_START;
            // TODO: reset reusable space
        }
    }
}
