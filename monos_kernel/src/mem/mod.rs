mod paging;
pub use paging::*;

use bootloader_api::info::MemoryRegions;
use core::{arch::asm, fmt, ops};
use spin::{Mutex, Once};

pub static MAPPER: Once<Mutex<Mapper>> = Once::new();
pub static FRAME_ALLOCATOR: Once<Mutex<FrameAllocator>> = Once::new();

pub fn init(physical_mem_offset: VirtualAddress, mem_map: &'static MemoryRegions) {
    // set up PAT to use write-combining write-through + cache-disabled pages (used for frame buffer)
    PAT::set(
        PAT::INDEX_WRITE_THROUGH | PAT::INDEX_CACHE_DISABLED,
        PAT::WRITE_COMBINING,
    );

    MAPPER.call_once(|| {
        let mapper = unsafe { Mapper::new(physical_mem_offset) };
        Mutex::new(mapper)
    });

    FRAME_ALLOCATOR.call_once(|| {
        let frame_allocator = FrameAllocator::new(mem_map);
        Mutex::new(frame_allocator)
    });
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VirtualAddress(u64);
impl VirtualAddress {
    #[inline]
    pub const fn new(address: u64) -> Self {
        VirtualAddress(address)
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn zero() -> Self {
        VirtualAddress(0)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        VirtualAddress(ptr as *const () as u64)
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    fn align(&self, align: u64) -> Self {
        Self::new(self.0 & !(align - 1))
    }

    #[inline]
    fn is_aligned(&self, align: u64) -> bool {
        self.0 & (align - 1) == 0
    }

    #[inline]
    #[allow(dead_code)]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl ops::Add<u64> for VirtualAddress {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        VirtualAddress(self.0 + rhs)
    }
}

impl ops::AddAssign<u64> for VirtualAddress {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VirtualAddress")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    #[inline]
    pub const fn new(address: u64) -> Self {
        let address = address % (1 << 52);

        // safety: we just truncated the address to 52 bits
        unsafe { Self::new_unchecked(address) }
    }

    #[inline]
    pub const unsafe fn new_unchecked(address: u64) -> Self {
        PhysicalAddress(address)
    }

    #[inline]
    fn align(&self, align: u64) -> Self {
        Self::new(self.0 & !(align - 1))
    }

    #[inline]
    fn is_aligned(&self, align: u64) -> bool {
        self.0 & (align - 1) == 0
    }

    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl ops::Add<u64> for PhysicalAddress {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        PhysicalAddress(self.0 + rhs)
    }
}

impl ops::AddAssign<u64> for PhysicalAddress {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PhysicalAddress")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

#[derive(Debug, Clone)]
#[repr(C, packed(2))]
pub struct DTPointer {
    pub limit: u16,
    pub base: VirtualAddress,
}

impl DTPointer {
    /// load the IDT at the pointer adress.
    ///
    /// safety: the pointer must point to a valid IDT and have the correct limit.
    pub unsafe fn load_idt(&self) {
        asm!("lidt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }

    /// load the GDT at the pointer adress.
    ///
    /// safety: the pointer must point to a valid GDT and have the correct limit.
    pub unsafe fn load_gdt(&self) {
        asm!("lgdt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }
}
