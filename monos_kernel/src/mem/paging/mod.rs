mod page;
pub use page::Page;

mod frame;
pub use frame::Frame;

mod page_table;
pub use page_table::{PageTable, PageTableFlags, PageTableIndex};

mod frame_allocator;
use frame_allocator::FrameAllocator;

mod mapper;
use mapper::{MapTo, Mapper};
pub use mapper::{MapToError, TranslateError, UnmapError};
mod mapping;
pub use mapping::Mapping;

mod pat;
pub use pat::PAT;

use crate::arch::registers::CR3;
use crate::mem::VirtualAddress;
use crate::utils::BitField;
use spin::{Mutex, Once};

use super::{physical_mem_offset, PhysicalAddress};
use bootloader_api::info::BootInfo;

static MAPPER: Once<Mutex<Mapper>> = Once::new();
static FRAME_ALLOCATOR: Once<Mutex<FrameAllocator>> = Once::new();

pub unsafe fn init(physical_mem_offset: VirtualAddress, boot_info: &BootInfo) {
    // set up PAT to use write-combining for write-through + cache-disabled pages (used for frame buffer)
    PAT::set(
        PAT::INDEX_WRITE_THROUGH | PAT::INDEX_CACHE_DISABLED,
        PAT::WRITE_COMBINING,
    );
    MAPPER.call_once(|| {
        let mapper = unsafe { Mapper::new(physical_mem_offset, active_level_4_table()) };
        Mutex::new(mapper)
    });
    FRAME_ALLOCATOR.call_once(|| {
        let start_frame = Frame::around(PhysicalAddress::new(
            boot_info.kernel_addr + boot_info.kernel_len,
        ));
        let frame_allocator = FrameAllocator::new(&boot_info.memory_regions, start_frame);
        Mutex::new(frame_allocator)
    });
}

pub fn translate_addr(virt: VirtualAddress) -> Result<PhysicalAddress, TranslateError> {
    MAPPER
        .get()
        .expect("memory hasn't been initialized yet")
        .lock()
        .translate_addr(virt)
}

pub unsafe fn map_to(
    page: &Page<PageSize4K>,
    frame: &Frame<PageSize4K>,
    flags: PageTableFlags,
) -> Result<(), MapToError> {
    MAPPER
        .get()
        .expect("memory hasn't been initialized yet")
        .lock()
        .map_to(page, frame, flags)
}

pub fn unmap(page: Page<PageSize4K>) -> Result<(), UnmapError> {
    MAPPER
        .get()
        .expect("memory hasn't been initialized yet")
        .lock()
        .unmap(page)
}

pub fn alloc_frame() -> Option<Frame<PageSize4K>> {
    FRAME_ALLOCATOR
        .get()
        .expect("memory hasn't been initialized yet")
        .lock()
        .allocate_frame()
}

pub trait PageSize: Copy {
    const SIZE: u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSize4K;
impl PageSize for PageSize4K {
    const SIZE: u64 = 4096;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSize2M;
impl PageSize for PageSize2M {
    const SIZE: u64 = 4096 * 512;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSize1G;
impl PageSize for PageSize1G {
    const SIZE: u64 = 4096 * 512 * 512;
}

impl VirtualAddress {
    fn page_offset(&self) -> u64 {
        self.as_u64().get_bits(0..12)
    }

    fn p1_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.as_u64().get_bits(12..21) as u16)
    }

    fn p2_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.as_u64().get_bits(21..30) as u16)
    }

    fn p3_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.as_u64().get_bits(30..39) as u16)
    }

    fn p4_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.as_u64().get_bits(39..48) as u16)
    }
}

pub fn active_level_4_table() -> &'static mut PageTable {
    let (l4_table, _) = CR3::read();
    let phys = l4_table.start_address();
    let virt = physical_mem_offset() + phys.as_u64();

    let ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *ptr }
}
