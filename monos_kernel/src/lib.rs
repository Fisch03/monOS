#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm_const)]

extern crate alloc;

mod acpi;
mod arch;
mod dev;
pub mod framebuffer;
pub mod fs;
mod gdt;
pub mod interrupts;
mod mem;
pub mod process;
pub mod serial;
pub mod syscall;
mod utils;

use bootloader_api::BootInfo;

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    interrupts::init_idt();
    syscall::init();

    interrupts::without_interrupts(|| {
        use arch::registers::CR4;
        use utils::BitField;
        let mut cr4 = CR4::read();

        cr4.set_bit(CR4::ENABLE_MACHINE_CHECK, true);
        cr4.set_bit(CR4::ENABLE_SSE, true);
        cr4.set_bit(CR4::ENABLE_UNMASKED_SSE, true);
        cr4.set_bit(CR4::TIME_STAMP_DISABLE, false);

        unsafe { CR4::write(cr4) };
    });

    // safety: the physical memory offset is valid since it was provided by the bootloader.
    // the bootloader config guarantees that the entire physical memory is mapped.
    println!("init mem");
    unsafe { mem::init(&boot_info) };

    println!("init fs");
    fs::init(boot_info);

    println!("init apic");
    interrupts::init_apic();
    println!("init acpi");
    acpi::init(boot_info);

    let fb = boot_info.framebuffer.take().unwrap();
    println!("init framebuffer");
    framebuffer::init(fb);

    println!("init devices");
    dev::init();
    interrupts::enable();
}
