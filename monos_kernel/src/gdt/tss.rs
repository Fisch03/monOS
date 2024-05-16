use crate::mem::VirtualAddress;
use core::mem::size_of;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    _reserved: u32,
    pub privilege_stack_table: [VirtualAddress; 3],
    _reserved2: u64,
    pub interrupt_stack_table: [VirtualAddress; 7],
    _reserved3: u64,
    _reserved4: u16,
    iomap_base: u16,
}

impl TaskStateSegment {
    #[inline]
    pub const fn new() -> Self {
        Self {
            privilege_stack_table: [VirtualAddress::zero(); 3],
            interrupt_stack_table: [VirtualAddress::zero(); 7],
            iomap_base: size_of::<Self>() as u16,

            _reserved: 0,
            _reserved2: 0,
            _reserved3: 0,
            _reserved4: 0,
        }
    }
}
