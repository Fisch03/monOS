use core::ops::{Bound, RangeBounds};

pub trait BitField {
    const LEN: usize;

    #[allow(dead_code)]
    fn get_bit(&self, index: usize) -> bool;
    #[allow(dead_code)]
    fn set_bit(&mut self, index: usize, value: bool);

    #[allow(dead_code)]
    fn get_bits(&self, range: impl RangeBounds<usize>) -> Self;
    #[allow(dead_code)]
    fn set_bits(&mut self, range: impl RangeBounds<usize>, value: Self);
}

macro_rules! impl_bitfield {
    ($($t:ty),*) => {
        $(
            impl BitField for $t {
                const LEN: usize = core::mem::size_of::<$t>() * 8;
                fn get_bit(&self, index: usize) -> bool {
                    assert!(index < Self::LEN);
                    *self & (1 << index) != 0
                }
                fn set_bit(&mut self, index: usize, value: bool) {
                    assert!(index < Self::LEN);
                    if value {
                        *self |= 1 << index;
                    } else {
                        *self &= !(1 << index);
                    }
                }
                fn get_bits(&self, range: impl RangeBounds<usize>) -> Self {
                    let start = match range.start_bound() {
                        Bound::Included(&start) => start,
                        Bound::Excluded(&start) => start + 1,
                        Bound::Unbounded => 0,
                    };
                    let end = match range.end_bound() {
                        Bound::Included(&end) => end,
                        Bound::Excluded(&end) => end - 1,
                        Bound::Unbounded => Self::LEN - 1,
                    };
                    assert!(start <= end && end < Self::LEN);
                    let mask = (1 << (end - start + 1)) - 1;
                    (self >> start) & mask
                }
                fn set_bits(&mut self, range: impl RangeBounds<usize>, value: Self) {
                    let start = match range.start_bound() {
                        Bound::Included(&start) => start,
                        Bound::Excluded(&start) => start + 1,
                        Bound::Unbounded => 0,
                    };
                    let end = match range.end_bound() {
                        Bound::Included(&end) => end,
                        Bound::Excluded(&end) => end - 1,
                        Bound::Unbounded => Self::LEN - 1,
                    };
                    assert!(start <= end && end < Self::LEN);
                    let mask = (1 << (end - start + 1)) - 1;
                    *self = (*self & !(mask << start)) | ((value & mask) << start);
                }
            }
        )*
    };
}

impl_bitfield!(u8, u16, u32, u64, u128, usize);
