use crate::ProcessId;
use core::marker::PhantomData;

pub trait MessageData
where
    Self: Sized,
{
    unsafe fn from_message(message: GenericMessage) -> Option<Self>;
    fn into_message(self) -> MessageType;
}

#[derive(Debug)]
pub enum MessageType {
    Scalar(u64, u64, u64, u64),
    Chunk {
        address: u64,
        size: u64,
        data: (u64, u64),
        is_mmapped: bool,
    },
}

impl MessageType {
    pub fn as_scalar(&self) -> Option<(u64, u64, u64, u64)> {
        match self {
            Self::Scalar(a, b, c, d) => Some((*a, *b, *c, *d)),
            _ => None,
        }
    }

    // safety: supplied type must match the type of the chunk
    pub unsafe fn as_chunk<T: Sized + 'static>(&self) -> Option<MemoryChunk<T>> {
        match self {
            Self::Chunk {
                address,
                size,
                is_mmapped,
                ..
            } => {
                if *is_mmapped {
                    return None;
                }

                debug_assert_eq!(size_of::<T>() as u64, *size); // sanity check
                let ptr = *address as *const T;
                let chunk = unsafe { MemoryChunk::new(ptr) };
                Some(chunk)
            }
            _ => None,
        }
    }

    pub fn as_mmapped_chunk<T: MMapSafe>(&self) -> Option<MemoryMappedChunk<T>> {
        match self {
            Self::Chunk {
                address,
                size,
                is_mmapped,
                ..
            } => {
                if !*is_mmapped {
                    return None;
                }
                debug_assert_eq!(size_of::<T>() as u64, *size); // sanity check
                let ptr = *address as *const T;
                let chunk = unsafe { MemoryMappedChunk::new(ptr) };
                Some(chunk)
            }
            _ => None,
        }
    }
}

pub struct MemoryChunk<T>
where
    T: Sized + 'static,
{
    pub address: u64,
    data: PhantomData<T>,
}

#[derive(Debug)]
pub struct MemoryMappedChunk<T: MMapSafe>(MemoryChunk<T>);

/// marker trait for types that can be safely mmapped.
/// this is safe to implement for any type that can not enter an invalid state from race conditions
/// where the scheduler could switch to another thread in the middle of a memory operation.
pub unsafe trait MMapSafe
where
    Self: Send + Sync + Sized + 'static,
{
}

impl<T> MemoryChunk<T>
where
    T: Sized + 'static,
{
    // safety: should only be called from the kernel on a correctly mapped memory chunk
    pub unsafe fn new(ptr: *const T) -> Self {
        Self {
            address: ptr as u64,
            data: PhantomData,
        }
    }

    pub fn size(&self) -> u64 {
        size_of::<T>() as u64
    }

    pub fn as_message(&self, data1: u64, data2: u64) -> MessageType {
        MessageType::Chunk {
            address: self.address,
            size: self.size(),
            data: (data1, data2),
            is_mmapped: false,
        }
    }
}

impl<T> MemoryChunk<T>
where
    T: MMapSafe,
{
    pub fn make_mmapped(self) -> MemoryMappedChunk<T> {
        MemoryMappedChunk(self)
    }
}

impl<T> MemoryMappedChunk<T>
where
    T: MMapSafe,
{
    // safety: should only be called from the kernel on a correctly mapped memory chunk
    pub unsafe fn new(ptr: *const T) -> Self {
        Self(MemoryChunk::new(ptr))
    }

    pub fn as_message(&self, data1: u64, data2: u64) -> MessageType {
        MessageType::Chunk {
            address: self.0.address,
            size: self.0.size(),
            data: (data1, data2),
            is_mmapped: true,
        }
    }
}

impl<T> Clone for MemoryMappedChunk<T>
where
    T: MMapSafe,
{
    fn clone(&self) -> Self {
        unsafe { MemoryMappedChunk::new(self.0.address as *const T) }
    }
}

impl<T> core::ops::Deref for MemoryChunk<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.address as *const T) }
    }
}

impl<T> core::ops::Deref for MemoryMappedChunk<T>
where
    T: MMapSafe,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> core::ops::DerefMut for MemoryChunk<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.address as *mut T) }
    }
}

impl<T> core::ops::DerefMut for MemoryMappedChunk<T>
where
    T: MMapSafe,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<T> core::fmt::Debug for MemoryChunk<T>
where
    T: Sized + core::fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use core::ops::Deref;
        f.debug_struct("MemoryChunk")
            .field("address", &self.address)
            .field("content", &self.deref())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, packed)]
pub struct ChannelHandle {
    pub target_process: ProcessId,
    pub target_channel: u16,
    pub own_channel: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartialSendChannelHandle {
    pub target_process: ProcessId,
    pub target_channel: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartialReceiveChannelHandle {
    pub own_channel: u16,
}

impl ChannelHandle {
    pub fn new(other_process: ProcessId, other_channel: u16, own_channel: u16) -> Self {
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

    #[cfg(feature = "userspace")]
    pub fn send<T: MessageData>(&self, data: T) {
        crate::syscall::send(*self, data);
    }

    #[cfg(feature = "userspace")]
    pub unsafe fn receive<T: MessageData>(&self) -> Option<T> {
        T::from_message(crate::syscall::receive(*self)?)
    }
}

impl PartialSendChannelHandle {
    pub fn new(other_process: ProcessId, other_channel: u16) -> Self {
        Self {
            target_process: other_process,
            target_channel: other_channel,
        }
    }
}

impl PartialEq<ChannelHandle> for PartialSendChannelHandle {
    fn eq(&self, other: &ChannelHandle) -> bool {
        let other_process = other.target_process;
        self.target_process == other_process && self.target_channel == other.target_channel
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
    // Limited(NonZeroU64), // TODO: implement limited channels
}

// impl From<u64> for ChannelLimit {
//     fn from(limit: u64) -> Self {
//         if limit == 0 {
//             Self::Unlimited
//         } else {
//             Self::Limited(NonZeroU64::new(limit).unwrap())
//         }
//     }
// }

impl From<ChannelLimit> for u64 {
    fn from(limit: ChannelLimit) -> Self {
        match limit {
            ChannelLimit::Unlimited => 0,
            // ChannelLimit::Limited(limit) => limit.get(),
        }
    }
}

#[derive(Debug)]
pub struct GenericMessage {
    pub sender: PartialSendChannelHandle,
    pub data: MessageType,
}

impl MessageData for GenericMessage {
    unsafe fn from_message(message: GenericMessage) -> Option<Self> {
        Some(message)
    }

    fn into_message(self) -> MessageType {
        self.data
    }
}
