use core::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelHandle(u32);
impl ChannelHandle {
    pub fn new(handle: u32) -> Self {
        Self(handle)
    }
}

impl From<ChannelHandle> for u32 {
    fn from(handle: ChannelHandle) -> Self {
        handle.0
    }
}

pub enum ChannelLimit {
    Unlimited,
    Limited(NonZeroU64),
}

impl From<u64> for ChannelLimit {
    fn from(limit: u64) -> Self {
        if limit == 0 {
            Self::Unlimited
        } else {
            Self::Limited(NonZeroU64::new(limit).unwrap())
        }
    }
}

impl From<ChannelLimit> for u64 {
    fn from(limit: ChannelLimit) -> Self {
        match limit {
            ChannelLimit::Unlimited => 0,
            ChannelLimit::Limited(limit) => limit.get(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: u64,
    pub handle: ChannelHandle,
    pub data: (u64, u64, u64, u64),
}
