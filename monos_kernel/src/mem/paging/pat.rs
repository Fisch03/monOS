use crate::arch::registers::MSR;
use crate::utils::BitField;

const IA32_PAT_MSR: u32 = 0x277;

pub struct PAT;

#[allow(dead_code)]
impl PAT {
    pub const UNCACHEABLE: u64 = 0;
    pub const WRITE_COMBINING: u64 = 1;
    pub const WRITE_THROUGH: u64 = 4;
    pub const WRITE_PROTECTED: u64 = 5;
    pub const WRITE_BACK: u64 = 6;
    pub const UNCACHED: u64 = 7;

    pub const INDEX_WRITE_THROUGH: usize = 1 << 0;
    pub const INDEX_CACHE_DISABLED: usize = 1 << 1;
    pub const INDEX_PAT: usize = 1 << 2;

    pub fn set(index: usize, value: u64) {
        assert!(index < 8, "PAT index out of bounds");
        assert!(value < 8, "PAT value out of bounds");

        let ia32_pat = MSR::new(IA32_PAT_MSR);

        let mut current_pat = unsafe { ia32_pat.read() };
        let range = index * 8..(index * 8) + 4;
        current_pat.set_bits(range, value);

        unsafe {
            ia32_pat.write(value);
        }
    }
}
