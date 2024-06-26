use core::num::NonZeroU64;

pub trait MessageData
where
    Self: Sized,
{
    unsafe fn from_message(message: &Message) -> Option<Self>;
    fn into_message(self) -> (u64, u64, u64, u64);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, packed)]
pub struct ChannelHandle {
    pub target_process: u32,
    pub target_channel: u16,
    pub own_channel: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartialSendChannelHandle {
    pub target_process: u32,
    pub target_channel: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartialReceiveChannelHandle {
    pub own_channel: u16,
}

impl ChannelHandle {
    pub fn new(other_process: u32, other_channel: u16, own_channel: u16) -> Self {
        Self {
            target_process: other_process,
            target_channel: other_channel,
            own_channel,
        }
    }

    pub fn from_parts(
        send_part: PartialSendChannelHandle,
        recv_part: PartialReceiveChannelHandle,
    ) -> Self {
        Self {
            target_process: send_part.target_process,
            target_channel: send_part.target_channel,
            own_channel: recv_part.own_channel,
        }
    }

    pub fn send_part(&self) -> PartialSendChannelHandle {
        PartialSendChannelHandle {
            target_process: self.target_process,
            target_channel: self.target_channel,
        }
    }

    pub fn recv_part(&self) -> PartialReceiveChannelHandle {
        PartialReceiveChannelHandle {
            own_channel: self.own_channel,
        }
    }
}

impl PartialSendChannelHandle {
    pub fn new(other_process: u32, other_channel: u16) -> Self {
        Self {
            target_process: other_process,
            target_channel: other_channel,
        }
    }
}

impl PartialEq<ChannelHandle> for PartialSendChannelHandle {
    fn eq(&self, other: &ChannelHandle) -> bool {
        self.target_process == other.target_process && self.target_channel == other.target_channel
    }
}

impl From<ChannelHandle> for PartialSendChannelHandle {
    fn from(handle: ChannelHandle) -> Self {
        Self {
            target_process: handle.target_process,
            target_channel: handle.target_channel,
        }
    }
}

impl PartialReceiveChannelHandle {
    pub fn new(own_channel: u16) -> Self {
        Self { own_channel }
    }
}

impl From<ChannelHandle> for PartialReceiveChannelHandle {
    fn from(handle: ChannelHandle) -> Self {
        Self {
            own_channel: handle.own_channel,
        }
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
    pub sender: PartialSendChannelHandle,
    pub data: (u64, u64, u64, u64),
}

impl MessageData for Message {
    unsafe fn from_message(message: &Message) -> Option<Self> {
        Some(message.clone())
    }

    fn into_message(self) -> (u64, u64, u64, u64) {
        self.data
    }
}
