use core::{arch::asm, fmt, ops};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VirtualAddress(u64);
impl VirtualAddress {
    #[inline]
    pub const fn new(address: u64) -> Self {
        VirtualAddress(address)
    }

    #[inline]
    pub const fn zero() -> Self {
        VirtualAddress(0)
    }

    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        VirtualAddress(ptr as *const () as u64)
    }

    #[inline]
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
        write!(f, "{:#x}", self.0)
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
