use core::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(packed)]
pub struct ChannelHandle {
    pub thread: u32,
    pub channel: u16,
}

impl ChannelHandle {
    pub fn new(thread: u32, channel: u16) -> Self {
        Self { thread, channel }
    }

    pub fn thread(&self) -> u32 {
        self.thread
    }

    pub fn channel(&self) -> u16 {
        self.channel
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
    pub data: (u64, u64, u64, u64),
}
