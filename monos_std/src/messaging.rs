use alloc::string::String;

pub struct Message {
    sender: u64,
    port: String,
    data: (u64, u64, u64, u64),
}
