use super::{
    map_to, unmap, Frame, MapToError, Page, PageTableFlags, PhysicalAddress, VirtualAddress,
};

use core::sync::atomic::{AtomicU64, Ordering};
use core::{fmt, marker::PhantomData, mem::size_of, ops};

static NEXT_VIRT_ADDR: AtomicU64 = AtomicU64::new(crate::MAPPING_START.as_u64());
fn alloc_vmem(size: u64) -> VirtualAddress {
    let size = (size + 4096 - 1) & !(4096 - 1);
    let addr = NEXT_VIRT_ADDR.fetch_add(size, Ordering::Relaxed);
    VirtualAddress::new(addr)
}

pub struct Mapping<T> {
    start_addr: VirtualAddress,
    start_page: Page,

    size: u64,
    mapped_size: u64,

    end_page: Page,
    end_frame: Frame,

    mapped_type: PhantomData<T>,
}

impl<T> Mapping<T> {
    /// map the physical address to the given virtual address
    ///
    /// unsafe: the physical address needs to point to the given structure and there needs to be
    /// enough room in the virtual address space
    pub unsafe fn new(phys: PhysicalAddress, size: usize) -> Result<Self, MapToError> {
        let virt = alloc_vmem(size as u64);

        let start_frame = Frame::around(phys);
        let start_page = Page::around(virt);

        let end_page = Page::around(virt + size_of::<T>() as u64);

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let mut frame = start_frame;
        let mut page = start_page;
        loop {
            unsafe { map_to(&page, &frame, flags) }?;

            if page == end_page {
                break;
            }

            page = page.next();
            frame = frame.next();
        }

        let start_addr = page.start_address() + phys.offset_in_page();

        Ok(Self {
            start_addr,
            start_page,
            size: size_of::<T>() as u64,
            mapped_size: end_page.end_address() - start_page.start_address(),

            end_page: page,
            end_frame: frame,

            mapped_type: PhantomData,
        })
    }

    #[inline]
    pub unsafe fn cast<U>(self) -> Mapping<U> {
        let old = core::mem::ManuallyDrop::new(self);

        if size_of::<U>() > old.mapped_size as usize {
            panic!("new type does not fit in the mapping");
        }

        Mapping {
            start_addr: old.start_addr,
            start_page: old.start_page,

            mapped_size: old.mapped_size,
            size: old.size.max(size_of::<U>() as u64),

            end_page: old.end_page,
            end_frame: old.end_frame,

            mapped_type: PhantomData,
        }
    }

    pub unsafe fn extend(&mut self, additional: u64) {
        self.size += additional as u64;

        let end_page = Page::around(self.start_addr + self.size);

        if end_page == self.end_page {
            return;
        }
        self.end_page = self.end_page.next();
        self.end_frame = self.end_frame.next();

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        loop {
            map_to(&self.end_page, &self.end_frame, flags).expect("failed to extend mapping");
            if self.end_page == end_page {
                break;
            }
            self.end_page = self.end_page.next();
            self.end_frame = self.end_frame.next();
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn start_addr(&self) -> VirtualAddress {
        self.start_addr
    }

    #[inline]
    #[allow(dead_code)]
    pub fn end_addr(&self) -> VirtualAddress {
        self.start_addr + self.size
    }

    #[inline]
    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl<T: fmt::Debug> fmt::Debug for Mapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ops::Deref;

        f.debug_struct("Mapping")
            .field("start_addr", &self.start_addr)
            .field("start_page", &self.start_page)
            .field("end_page", &self.end_page)
            .field("end_frame", &self.end_frame)
            .field("size", &self.size)
            .field("mapped_size", &self.mapped_size)
            .field("inner", self.deref())
            .finish()
    }
}

impl<T> ops::Deref for Mapping<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // safety:: the caller of the constructor guaranteed the address is valid
        unsafe { &*(self.start_addr.as_ptr() as *const T) }
    }
}

impl<T> ops::DerefMut for Mapping<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // safety:: the caller of the constructor guaranteed the address is valid
        unsafe { &mut *(self.start_addr.as_mut_ptr() as *mut T) }
    }
}

impl<T> Drop for Mapping<T> {
    fn drop(&mut self) {
        let mut page = self.start_page;

        loop {
            unmap(&page).expect("failed to unmap memory");

            if page == self.end_page {
                break;
            }

            page = page.next()
        }
    }
}
