use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;

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
    const fn new(size: usize) -> Self {
        ReusableSpace { next: None, size }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size - 1
    }

    fn is_suitable(&self, layout: &Layout) -> bool {
        // block is suitable if aligned address withing block + size is also within block
        let aligned_start_addr = align_up(self.start_addr(), layout);
        let aligned_end_addr = aligned_start_addr + layout.size() - 1;

        if self.end_addr() < aligned_end_addr {
            return false;
        }
        let excess_size = self.end_addr() - aligned_end_addr;

        // excess after aligning up (simulating putting an entry of free memory here)
        let aligned_excess = align_up(aligned_end_addr, &Layout::new::<ReusableSpace>());
        let aligned_excess_size = aligned_excess + mem::size_of::<ReusableSpace>();

        // if there is not enough excess space accounting for aligns, go find another space
        if excess_size > 0 && excess_size < aligned_excess_size {
            return false;
        }

        aligned_end_addr <= self.end_addr()
    }
}

pub struct LinkedListAllocator {
    alloc_refs: usize,
    alloc_start: usize,
    root_reusable: ReusableSpace,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            alloc_refs: 0,
            alloc_start: HEAP_START,
            root_reusable: ReusableSpace::new(0), // root is always a dummy value
        }
    }
    /// Returns Some((addr, size)) if there is a suitable memory slot
    fn find_suitable_reusable_slot(&mut self, layout: &Layout) -> Option<(usize, usize)> {
        // we don't use the root since it is always a dummy value
        let mut current = &mut self.root_reusable;

        while let Some(ref mut region) = current.next {
            if region.is_suitable(layout) {
                let next = region.next.take();
                let ret = Some((region.start_addr(), region.size));
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        log!(Level::Debug, "allocating {} bytes", layout.size());

        let reusable_slot = self.lock().find_suitable_reusable_slot(&layout);
        let alloc_addr = match reusable_slot {
            Some((start_addr, size)) => {
                log!(Level::Debug, "Found reusable slot");
                let excess_size = size - layout.size();
                // if there is excess, create new ReusableSpace entry
                // (checks were already done by is_suitable)
                if excess_size > 0 {
                    // create new space
                    let mut space = ReusableSpace::new(layout.size());
                    // make space point to next of root
                    space.next = self.lock().root_reusable.next.take();
                    // write space struct at pointer position
                    let node_ptr = start_addr as *const i32 as *mut ReusableSpace;
                    node_ptr.write(space);
                    // make head pont to new space (root -> space -> old_next_of_head)
                    self.lock().root_reusable.next = Some(&mut *node_ptr);
                }

                start_addr
            }
            None => {
                // alloc at memory end
                //
                // first we align memory
                let alloc_start = self.lock().alloc_start;
                let aligned_start_addr = align_up(alloc_start, &layout);

                // then we check if there's enough space after aligning
                let new_alloc_end = aligned_start_addr + layout.size();
                if new_alloc_end > HEAP_END {
                    panic!("Not enough space on heap. New alloc end is past actual heap end.");
                }

                self.lock().alloc_start = aligned_start_addr + layout.size();
                aligned_start_addr
            }
        };

        // alloc space
        self.lock().alloc_refs += 1;

        // no need to map since init() already maps it all

        alloc_addr as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        log!(Level::Debug, "deallocating {} bytes", layout.size());
        // decrement alloc_refs
        self.lock().alloc_refs -= 1;

        // put in reusable space (obs: only put in reusable if size is > than size of
        // reusable entry, otherwise memory fuckers)
        let is_aligned = align_up(ptr as usize, &Layout::new::<ReusableSpace>()) == ptr as usize;
        if layout.size() >= mem::size_of::<ReusableSpace>() && is_aligned {
            // create new reusable node
            let mut space = ReusableSpace::new(layout.size());
            // make space point to next of root
            space.next = self.lock().root_reusable.next.take();
            // write space struct at pointer position
            let node_ptr = ptr as *mut ReusableSpace;
            log!(
                Level::Debug,
                "adding {} bytes of reusable space at addr {}",
                space.size,
                ptr as usize
            );
            node_ptr.write(space);
            // make head pont to new space (root -> space -> old_next_of_head)
            self.lock().root_reusable.next = Some(&mut *node_ptr);
        }

        if self.lock().alloc_refs == 0 {
            self.lock().alloc_refs = 0;
            self.lock().alloc_start = HEAP_START;
            // reset reusable space list
            self.lock().root_reusable.next = None;
        }
    }
}
