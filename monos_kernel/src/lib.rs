#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]

extern crate alloc;

mod acpi;
mod arch;
pub mod dev;
pub mod framebuffer;
pub mod fs;
pub mod gdt;
pub mod interrupts;
mod mem;
pub mod process;
pub mod serial;
pub mod syscall;
mod utils;

use bootloader_api::{config, BootInfo, BootloaderConfig};
use core::arch::asm;

const LOWER_HALF_END: u64 = 0x0000_8000_0000_0000;

use mem::VirtualAddress;
const FB_START: VirtualAddress = VirtualAddress::new(0xffff_900_000_000_000);
const HEAP_START: VirtualAddress = VirtualAddress::new(0xffff_a00_000_000_000);
const MAPPING_START: VirtualAddress = VirtualAddress::new(0xffff_b00_000_000_000);
const APIC_ADDR: VirtualAddress = VirtualAddress::new(0xffff_fff_000_000_000);

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(config::Mapping::Dynamic);
    config.mappings.dynamic_range_start = Some(0xffff_8000_0000_0000);

    // currently the frame allocator bitmap lives fully on the stack, so we need a bigger stack
    config.kernel_stack_size = 1024 * 1024;

    config
};

mod test {
    use monos_test::kernel_test;

    #[kernel_test]
    fn testing_works(_: &bootloader_api::BootInfo) -> bool {
        assert_eq!(1, 1);

        true
    }
}

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    interrupts::init_idt();
    syscall::init();

    interrupts::without_interrupts(|| {
        use arch::registers::CR4;
        use utils::BitField;
        let mut cr4 = CR4::read();

        cr4.set_bit(CR4::ENABLE_MACHINE_CHECK, true);
        // cr4.set_bit(CR4::ENABLE_SSE, true);
        // cr4.set_bit(CR4::ENABLE_UNMASKED_SSE, true);
        cr4.set_bit(CR4::TIME_STAMP_DISABLE, false);

        unsafe { CR4::write(cr4) };
    });

    println!("init mem");

    for region in boot_info
        .memory_regions
        .iter()
        .filter(|r| r.kind != bootloader_api::info::MemoryRegionKind::Usable)
    {
        crate::println!(
            "mem region: {:#x} - {:#x} {:?}",
            region.start,
            region.end,
            region.kind
        );
    }

    let usable_memory = boot_info
        .memory_regions
        .iter()
        .filter(|r| r.kind == bootloader_api::info::MemoryRegionKind::Usable)
        .fold(0, |acc, r| acc + r.end - r.start);
    let usable_mb = usable_memory / 1024 / 1024;
    crate::println!("found ~{}MB of usable memory", usable_mb);

    // safety: the physical memory offset is valid since it was provided by the bootloader.
    // the bootloader config guarantees that the entire physical memory is mapped.
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

use core::panic::PanicInfo;
pub fn kernel_panic(info: &PanicInfo) -> ! {
    interrupts::disable();

    unsafe { crate::process::CURRENT_PROCESS.force_write_unlock() };
    let current_proc = crate::process::CURRENT_PROCESS.read();
    if let Some(current_proc) = current_proc.as_ref() {
        println!(
            "in pid: {} ({})",
            current_proc.id().as_u32(),
            current_proc.name(),
        );
    } else {
        println!("in kernel");
    }

    if let Some(location) = info.location() {
        println!(
            "at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    println!("message: {}", info.message());

    if let Some(mut fb_guard) = framebuffer::get() {
        let _fb = unsafe { fb_guard.now_or_never() };

        // TODO

        fb_guard.submit_kernel_frame();
    }

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
