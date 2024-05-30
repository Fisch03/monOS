#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm_const)]

extern crate alloc;

mod acpi;
mod arch;
mod core_local;
mod dev;
mod gdt;
pub mod gfx;
pub mod interrupts;
mod mem;
pub mod process;
pub mod serial;
pub mod syscall;
mod utils;

use bootloader_api::BootInfo;

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    core_local::CoreLocal::init();

    gdt::init();
    interrupts::init_idt();
    syscall::init();

    // safety: the physical memory offset is valid since it was provided by the bootloader.
    // the bootloader config guarantees that the entire physical memory is mapped.
    unsafe { mem::init(&boot_info) };

    interrupts::init_apic();
    acpi::init(boot_info);

    let fb = boot_info.framebuffer.take().unwrap();
    gfx::init(fb);

    dev::init();
    interrupts::enable();
}
