use core::{fmt, ops};

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
