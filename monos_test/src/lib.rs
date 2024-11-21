#![no_std]

pub use derive::kernel_test;

type TestFn = fn(boot_info: &'static bootloader_api::BootInfo) -> bool;

pub struct Location {
    pub module: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

pub struct TestDescription {
    pub name: &'static str,
    pub test_fn: TestFn,
    pub location: Location,
}

#[linkme::distributed_slice]
pub static KERNEL_TESTS: [TestDescription] = [..];
