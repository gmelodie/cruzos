use bootloader::BootInfo;

use crate::{memory, util::Locked};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
pub const HEAP_END: usize = HEAP_START + HEAP_SIZE;

pub struct BumpAllocator {
    alloc_refs: usize,
    alloc_start: usize,
}

impl BumpAllocator {
    pub const fn new(boot_info: &BootInfo) -> Self {
        BumpAllocator {
            alloc_refs: 0,
            alloc_start: HEAP_START,
        }
    }
}

impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let new_heap_end = self.lock().alloc_start + layout.size();
        if new_heap_end > HEAP_END {
            panic!("Not enough space left to allocate");
        }

        // check if we can allocate layout.size()
        let alloc_addr = self.lock().alloc_start;
        let new_alloc_end = alloc_addr + layout.size();
        if new_alloc_end > HEAP_END {
            panic!("Not enough space on heap. New alloc end is past actual heap end.");
        }

        // alloc space
        *self.lock().alloc_refs += 1;
        *self.lock().alloc_start += layout.size();

        // map allocd pages
        // TODO: alloc the amount of pages we want, not just one
        // let page = Page::<Size4KiB>::containing_address(VirtAddr::new(alloc_addr));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let mut frame_allocator: unsafe { memory::FrameAllocator::new(&boot_info.memory_map) };
        for page in allocd_pages {
            memory::map_virt(page, flags, &mut frame_allocator);
        }

        (alloc_addr as *mut u8)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // decrement alloc_refs
        *self.lock().alloc_refs -= 1;
        if self.lock().alloc_refs == 0 {
            // unmap all heap memory
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end = VirtAddr::new(HEAP_END as u64);
            let heap_end_page = Page::containing_address(heap_end);

            for page in Page::range_inclusive(heap_start_page, heap_end_page) {
                memory::unmap_virt(page);
            }
        }
    }
}
