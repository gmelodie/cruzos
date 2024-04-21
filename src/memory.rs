use crate::prelude::*;
use bootloader::bootinfo::MemoryMap;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PhysFrame;
use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

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
    use x86_64::structures::paging::page_table::FrameError;
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
        let phys = entry_table_frame.start_address();
        let virt = to_mapped_mem(phys);
        let page_table_pointer: *mut PageTable = virt.as_mut_ptr();
        page_table = unsafe { &mut *page_table_pointer }; // unsafe: creating reference to raw pointer

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

fn create_page_table(layer: usize) -> &mut PageTable {
    if layer == 0 {
        return; // base case
    }
    // find free frame
    // TODO: alloc other page tables
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
    // TODO: finish implementing
    pub fn alloc_page(&mut self) -> VirtAddr {
        let frame = match self.allocate_frame() {
            Some(frame) => frame,
            None => panic!("Could not allocate frame"),
        };
        let mut l4 = unsafe { active_layer_4_page_table() };
        // let mut l3 =
        // 0. find unused frame in previously allocated l1
        // 1. if all l1 entries are used, allocate new l1
        // 2. if all l2 entries are used, allocate new l2
        // 3. if all l3 entries are used, allocate new l3
        // 4. if all l4 entries are used, error!! memory EXPLODING full
        // Obs: this should be a DFS
        //
        // after you found the l1, l2, l3 and l4 indexes: put the mapping into the l1 table
        // return address including l1, l2, l3 and l4 indexes as well as page offset
        // p1[page.p1_index()].set_frame(frame, flags);
        // addr = p1 + p2 + p3 + p4 + offset (offset is zero)
        // VirtAddr::new(addr.start_address())
        VirtAddr::new(0)
    }
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
