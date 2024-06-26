use crate::prelude::*;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use bootloader::BootInfo;
use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    frame::PhysFrame,
    page_table::{FrameError, PageTableEntry, PageTableFlags},
    Page, PageTable,
};
use x86_64::{PhysAddr, VirtAddr};

lazy_static! {
    // needs to be initialized with BootInfo (see init() fn on this file)
    pub static ref PHYSICAL_MEMORY_OFFSET: Mutex<VirtAddr> = Mutex::new(VirtAddr::new(0));
}

const PAGE_SIZE: usize = 4096;

pub fn init(boot_info: &BootInfo) {
    *PHYSICAL_MEMORY_OFFSET.lock() = VirtAddr::new(boot_info.physical_memory_offset);
}

/// Returns the address of the layer 4 page table in virtual memory
/// given the offset at which the mapping of physical to virtual memory is
pub unsafe fn active_layer_4_page_table() -> &'static mut PageTable {
    // first we get the frame struct of the frame containing
    // the layer 4 page table
    let (layer_4_table_frame, _) = Cr3::read();

    // we need the first address in the frame
    let phys = layer_4_table_frame.start_address();

    // then we add the mapping offset to get the virtual address to which
    // this physical address is mapped to (because we can only operate
    // on physical addresses)
    let virt = to_mapped_mem(phys);

    let page_table_pointer: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_pointer // unsafe: creating reference to raw pointer
}

/// Converts an virtual address to a physical one by
/// traversing the 4 layer page tables
pub unsafe fn virt2phys(addr: VirtAddr) -> Option<PhysAddr> {
    let page_table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    // get l4 page table frame
    let (mut entry_table_frame, _) = Cr3::read();
    let mut page_table;

    for idx in page_table_indexes {
        // convert frame to reference
        page_table = unsafe { frame_to_page_table(entry_table_frame) }; // unsafe: creating reference to raw pointer

        // access page table
        entry_table_frame = match page_table[idx].frame() {
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Found huge frame. Huge frames are not supported"),
            Ok(frame) => frame,
        };
    }

    // calculate physical address by adding page offset
    Some(entry_table_frame.start_address() + addr.page_offset().into())
}

fn to_mapped_mem(phys: PhysAddr) -> VirtAddr {
    *PHYSICAL_MEMORY_OFFSET.lock() + phys.as_u64()
}

unsafe fn frame_to_page_table(frame: PhysFrame) -> &'static mut PageTable {
    // convert frame to reference
    let phys = frame.start_address();
    let virt = to_mapped_mem(phys);
    let page_table_pointer: *mut PageTable = virt.as_mut_ptr();
    let page_table = unsafe { &mut *page_table_pointer }; // unsafe: creating reference to raw pointer
    page_table
}

pub struct FrameAllocator<'frame_life> {
    memory_map: &'frame_life MemoryMap,
    next: usize,
}

impl<'frame_life> FrameAllocator<'frame_life> {
    pub unsafe fn new(memory_map: &'frame_life MemoryMap) -> Self {
        FrameAllocator {
            memory_map,
            next: 0,
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + 'frame_life {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(PAGE_SIZE));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }

    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Maps a page (in virtual memory space) to a usable frame (in physical memory space).
/// Frame to be mapped to page is any usable frame we can find.
/// We use allocator if we need to create new pages.
pub fn map_virt(page: Page, flags: PageTableFlags, frame_allocator: &mut FrameAllocator) {
    let frame = match frame_allocator.allocate_frame() {
        Some(frame) => frame,
        None => panic!("Could not allocate frame"),
    };
    let l4 = unsafe { active_layer_4_page_table() };
    let l3 = create_page_table(&mut l4[page.p4_index()], frame_allocator, flags);
    let l2 = create_page_table(&mut l3[page.p3_index()], frame_allocator, flags);
    let l1 = create_page_table(&mut l2[page.p2_index()], frame_allocator, flags);
    if !l1[page.p1_index()].is_unused() {
        panic!("Page already mapped");
    }
    l1[page.p1_index()].set_frame(frame, flags);
    // ensure we're using the newest mapping
    tlb::flush(page.start_address());
}

/// Unmaps a page (in virtual memory space) back to an unused frame (in physical memory space).
pub fn unmap_virt(page: Page) {
    let l4 = unsafe { active_layer_4_page_table() };

    let l3_entry = &l4[page.p4_index()];
    if l3_entry.is_unused() {
        panic!("Cannot unmap: page already unmapped");
    }
    let l3 = unsafe { frame_to_page_table(l3_entry.frame().unwrap()) };

    let l2_entry = &l3[page.p3_index()];
    if l2_entry.is_unused() {
        panic!("Cannot unmap: page already unmapped");
    }
    let l2 = unsafe { frame_to_page_table(l2_entry.frame().unwrap()) };

    let l1_entry = &l2[page.p2_index()];
    if l1_entry.is_unused() {
        panic!("Cannot unmap: page already unmapped");
    }
    let l1 = unsafe { frame_to_page_table(l1_entry.frame().unwrap()) };

    l1[page.p1_index()].set_unused();
    // ensure we're using the newest mapping
    tlb::flush(page.start_address());
}

/// Ensures a page table exists given a page table entry and returns it
fn create_page_table(
    entry: &mut PageTableEntry,
    frame_allocator: &mut FrameAllocator,
    flags: PageTableFlags,
) -> &'static mut PageTable {
    let created: bool;

    if entry.is_unused() {
        log!(
            Level::Debug,
            "Allocating frame for entry {:?}",
            entry.addr()
        );
        let frame = match frame_allocator.allocate_frame() {
            Some(frame) => frame,
            None => panic!("Could not allocate frame"),
        };
        entry.set_frame(frame, flags);
        created = true;
    } else {
        log!(
            Level::Debug,
            "Page table already exists at entry {:?}",
            entry.addr()
        );
        if !flags.is_empty() && !entry.flags().contains(flags) {
            entry.set_flags(entry.flags() | flags);
        }
        created = false;
    }

    let page_table = unsafe { frame_to_page_table(entry.frame().unwrap()) };

    if created {
        page_table.zero();
    }

    page_table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_virt2phys() {
        // vga identity should be the same
        let expected = PhysAddr::new(0xb8000);
        let actual = unsafe { virt2phys(VirtAddr::new(0xb8000)).unwrap() };
        assert_eq!(actual, expected);
    }
}
