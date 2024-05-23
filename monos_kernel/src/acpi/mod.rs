mod rsdp;
use rsdp::RSDP;

use bootloader_api::info::BootInfo;

use crate::{dbg, mem::PhysicalAddress};

pub fn init(boot_info: &BootInfo) {
    let rsdp_phys = boot_info.rsdp_addr.as_ref().expect("no rsdp table found");
    let rsdp = RSDP::new(PhysicalAddress::new(*rsdp_phys));
    dbg!(unsafe { &*rsdp.as_ref().unwrap().as_ptr::<RSDP>() });
}
