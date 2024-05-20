use super::BitField;
use core::fmt;
use core::iter::IntoIterator;

#[repr(transparent)]
pub struct BitArray<const NUM_BYTES: usize> {
    data: [u8; NUM_BYTES],
}

impl<const NUM_BYTES: usize> BitArray<NUM_BYTES> {
    #[inline]
    pub fn new() -> Self {
        Self {
            data: [0; NUM_BYTES],
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        let byte = self.data.get(index / 8)?;
        Some(byte.get_bit(index % 8))
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        let byte = self.data.get_mut(index / 8);
        match byte {
            None => return,
            Some(byte) => byte.set_bit(index % 8, value),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        NUM_BYTES * 8
    }

    #[inline]
    pub fn iter(&self) -> BitArrayIter<'_, NUM_BYTES> {
        self.into_iter()
    }
}

impl<const NUM_BYTES: usize> fmt::Debug for BitArray<NUM_BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BitArray")
            .field(&format_args!("{:?}", &self.data))
            .finish()
    }
}

pub struct BitArrayIter<'a, const NUM_BYTES: usize> {
    array: &'a BitArray<NUM_BYTES>,
    index: usize,
}

impl<const NUM_BYTES: usize> Iterator for BitArrayIter<'_, NUM_BYTES> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        let bit = self.array.get(self.index)?;
        self.index += 1;
        Some(bit)
    }
}

impl<'a, const NUM_BYTES: usize> IntoIterator for &'a BitArray<NUM_BYTES> {
    type Item = bool;
    type IntoIter = BitArrayIter<'a, NUM_BYTES>;
    fn into_iter(self) -> Self::IntoIter {
        BitArrayIter {
            array: &self,
            index: 0,
        }
    }
}
