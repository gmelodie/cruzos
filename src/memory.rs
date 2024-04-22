use crate::prelude::*;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::instructions::tlb;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    frame::PhysFrame,
    page_table::{FrameError, PageTableEntry, PageTableFlags},
    Page, PageTable,
};
use x86_64::{PhysAddr, VirtAddr};

lazy_static! {
    // needs to be initialized with BootInfo
    pub static ref PHYSICAL_MEMORY_OFFSET: Mutex<VirtAddr> = Mutex::new(VirtAddr::new(0));
}

const PAGE_SIZE: usize = 4096;

pub fn init(physical_memory_offset: u64) {
    *PHYSICAL_MEMORY_OFFSET.lock() = VirtAddr::new(physical_memory_offset);
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

struct Allocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl Allocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Allocator {
            memory_map,
            next: 0,
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
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

/// Maps a given virtual address to a usable frame
pub fn map_to(allocator: &mut Allocator, addr: VirtAddr) {
    let page = Page::containing_address(addr);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let l4 = unsafe { active_layer_4_page_table() };
    let l3 = create_page_table(&mut l4[page.p4_index()], allocator, flags);
    let l2 = create_page_table(&mut l3[page.p3_index()], allocator, flags);
    let l1 = create_page_table(&mut l2[page.p2_index()], allocator, flags);
    if !l1[page.p1_index()].is_unused() {
        panic!("Page already mapped");
    }
    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => panic!("Could not allocate frame"),
    };
    l1[page.p1_index()].set_frame(frame, flags);
    // ensure we're using the newest mapping
    tlb::flush(addr);
}

/// Ensures a page table exists given a page table entry and returns it
fn create_page_table(
    entry: &mut PageTableEntry,
    allocator: &mut Allocator,
    flags: PageTableFlags,
) -> &'static mut PageTable {
    let created: bool;

    if entry.is_unused() {
        let frame = match allocator.allocate_frame() {
            Some(frame) => frame,
            None => panic!("Could not allocate frame"),
        };
        entry.set_frame(frame, flags);
        created = true;
    } else {
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
