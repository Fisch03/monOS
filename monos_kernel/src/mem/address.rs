use core::{fmt, ops};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct VirtualAddress(u64);
impl VirtualAddress {
    #[inline]
    pub const fn new(address: u64) -> Self {
        let truncated_address = (((address << 16) as i64) >> 16) as u64;
        if truncated_address != address {
            panic!("virtual address is not a canonical address");
        }
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
    pub unsafe fn read<T>(&self) -> T
    where
        T: Copy,
    {
        *(self.as_ptr::<T>())
    }

    #[inline]
    pub unsafe fn write<T>(&self, value: T)
    where
        T: Copy,
    {
        *(self.as_mut_ptr::<T>()) = value;
    }

    #[inline]
    pub fn align(&self, align: u64) -> Self {
        Self::new(self.0 & !(align - 1))
    }

    #[inline]
    pub fn align_up(&self, align: u64) -> Self {
        Self::new((self.0 + align - 1) & !(align - 1))
    }

    #[inline]
    pub fn is_aligned(&self, align: u64) -> bool {
        self.0 & (align - 1) == 0
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
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

impl ops::Sub<u64> for VirtualAddress {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self {
        VirtualAddress(self.0 - rhs)
    }
}

impl ops::SubAssign<u64> for VirtualAddress {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}

impl ops::Sub<VirtualAddress> for VirtualAddress {
    type Output = u64;
    fn sub(self, rhs: VirtualAddress) -> u64 {
        self.0 - rhs.0
    }
}

impl ops::SubAssign<VirtualAddress> for VirtualAddress {
    fn sub_assign(&mut self, rhs: VirtualAddress) {
        self.0 -= rhs.0;
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VirtualAddress")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    #[inline]
    pub const fn new(address: u64) -> Self {
        let new_address = address % (1 << 52);
        if new_address != address {
            panic!("physical address is too large");
        }

        // safety: we just truncated the address to 52 bits
        unsafe { Self::new_unchecked(address) }
    }

    #[inline]
    pub const fn try_new(address: u64) -> Option<Self> {
        let new_address = address % (1 << 52);
        if new_address != address {
            return None;
        }
        Some(unsafe { Self::new_unchecked(address) })
    }

    #[inline]
    pub const unsafe fn new_unchecked(address: u64) -> Self {
        PhysicalAddress(address)
    }

    #[inline]
    pub fn align(&self, align: u64) -> Self {
        Self::new(self.0 & !(align - 1))
    }

    #[inline]
    pub fn align_up(&self, align: u64) -> Self {
        Self::new((self.0 + align - 1) & !(align - 1))
    }

    #[inline]
    pub fn is_aligned(&self, align: u64) -> bool {
        self.0 & (align - 1) == 0
    }

    #[inline]
    pub const fn offset_in_page(&self) -> u64 {
        self.0 & 0xfff
    }

    #[inline(always)]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
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
