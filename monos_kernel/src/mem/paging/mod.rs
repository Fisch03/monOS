mod page;
pub use page::Page;

mod frame;
pub use frame::Frame;

mod page_table;
pub use page_table::{PageTable, PageTableFlags, PageTableIndex};

mod frame_allocator;
use frame_allocator::FrameAllocator;

mod mapper;
pub use mapper::{MapTo, Mapper};
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
        let mut frame_allocator = FrameAllocator::new(&boot_info.memory_regions, start_frame);

        // if let Some(ramdisk_addr) = boot_info.ramdisk_addr.as_ref() {
        //     let ramdisk_phys = translate_addr(VirtualAddress::new(*ramdisk_addr))
        //         .expect("failed to translate ramdisk address");
        //
        //     frame_allocator.reserve_range(ramdisk_phys, boot_info.ramdisk_len as usize);
        // }

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

pub fn unmap(page: &Page<PageSize4K>) -> Result<(), UnmapError> {
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

pub fn empty_page_table() -> (*mut PageTable, Frame) {
    let page_table_frame = alloc_frame().expect("failed to alloc frame for process page table");
    let page_table_page = Page::around(super::alloc_vmem(4096));
    unsafe {
        map_to(
            &page_table_page,
            &page_table_frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        )
        .expect("failed to map page table page");
    };

    let page_table_ptr: *mut PageTable = page_table_page.start_address().as_mut_ptr();

    unsafe {
        (*page_table_ptr).clear();
    }

    (page_table_ptr, page_table_frame)
}

pub fn copy_pagetable(source_l4: &PageTable, target_l4: &mut PageTable) {
    fn copy_recursive(
        physical_mem_offset: VirtualAddress,
        source: &PageTable,
        dest: &mut PageTable,
        level: u16,
    ) {
        for (i, entry) in source.iter().enumerate() {
            if entry.is_present() {
                if level == 1 || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    unsafe {
                        dest[i].set_addr(entry.addr());
                        dest[i].set_flags(&entry.flags());
                    }
                } else {
                    let (new_page_table, new_frame) = empty_page_table();
                    let new_dest = unsafe { &mut *new_page_table };

                    unsafe {
                        dest[i].set_frame(&new_frame);
                        dest[i].set_flags(&entry.flags());
                    }

                    let new_source = {
                        let virt = physical_mem_offset + entry.addr().as_u64();
                        unsafe { &*virt.as_ptr() }
                    };

                    copy_recursive(physical_mem_offset, new_source, new_dest, level - 1);
                }
            }
        }
    }

    copy_recursive(physical_mem_offset(), source_l4, target_l4, 4);
}

pub fn create_user_demand_pages(
    mapper: &mut Mapper,
    start: VirtualAddress,
    size: u64,
) -> Result<(), MapToError> {
    let initial_frame = alloc_frame().ok_or(MapToError::OutOfMemory)?;

    let mut page = Page::around(start);
    let end_page: Page<PageSize4K> = Page::around(page.start_address() + size).next();
    crate::dbg!(end_page);

    // make the first page usable normally
    unsafe {
        mapper.map_to(
            &page,
            &initial_frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        )?;
    }

    page = page.next();

    // map the rest to the same frame, but with only read permissions. they will trigger a page
    // fault when written which allows us to allocate the frame lazily.
    loop {
        unsafe {
            mapper.map_to_with_parent_flags(
                &page,
                &initial_frame,
                PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE,
            )?;
        }

        if page == end_page {
            break;
        }

        page = page.next();
    }

    Ok(())
}

pub fn alloc_demand_page(virt: VirtualAddress) -> Result<(), &'static str> {
    let mut table = active_level_4_table();
    for index in [virt.p4_index(), virt.p3_index(), virt.p2_index()] {
        let entry = &mut table[index];
        table = unsafe { &mut *(physical_mem_offset() + entry.addr().as_u64()).as_mut_ptr() };
    }
    let entry = &mut table[virt.p1_index()];

    if entry.flags() != (PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE) {
        return Err("page is not demand-allocated");
    }

    let frame = alloc_frame().ok_or("failed to allocate frame for demand page")?;

    unsafe {
        entry.set_addr(frame.start_address());
        entry.set_flags(
            &(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE),
        );
    }

    Ok(())
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
