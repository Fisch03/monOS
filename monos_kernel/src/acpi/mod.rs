mod rsdp;
use rsdp::RSDP;

mod sdt;
use sdt::ACPIRoot;

mod tables;

use bootloader_api::info::BootInfo;

use crate::{dbg, mem::PhysicalAddress};

pub fn init(boot_info: &BootInfo) {
    let rsdp_phys = boot_info.rsdp_addr.as_ref().expect("no rsdp table found");
    let rsdp = RSDP::new(PhysicalAddress::new(*rsdp_phys)).expect("failed to map rsdp");

    let acpi_root = ACPIRoot::new(rsdp);

    let madt = acpi_root
        .get_table::<tables::MADT>()
        .expect("no MADT table found");

    madt.get_entries::<tables::madt::ProcessorLocalAPIC>()
        .for_each(|entry| {
            dbg!(entry);
        });

    madt.get_entries::<tables::madt::IOAPIC>()
        .for_each(|entry| {
            dbg!(entry);
        });
}
