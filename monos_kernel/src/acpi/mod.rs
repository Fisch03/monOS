mod rsdp;
use rsdp::RSDP;

mod sdt;
use sdt::ACPIRoot;

pub mod tables;

use bootloader_api::info::BootInfo;

use crate::mem::PhysicalAddress;
use spin::Once;

pub static ACPI_ROOT: Once<ACPIRoot> = Once::new();

pub fn init(boot_info: &BootInfo) {
    ACPI_ROOT.call_once(|| {
        let rsdp_phys = boot_info.rsdp_addr.as_ref().expect("no rsdp table found");
        let rsdp = RSDP::new(PhysicalAddress::new(*rsdp_phys)).expect("failed to map rsdp");
        ACPIRoot::new(rsdp)
    });
}
