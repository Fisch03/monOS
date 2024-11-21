#![no_std]
#![no_main]

use bootloader_api::BootInfo;

use monos_kernel::*;
use monos_test::*;

extern crate alloc;

bootloader_api::entry_point!(test_kernel_main, config = &BOOTLOADER_CONFIG);

static mut CURRENT_TEST: Option<&'static TestDescription> = None;

fn test_kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // monos_kernel::kernel_init(boot_info);

    gdt::init();
    interrupts::init_idt();

    println!("running {} kernel tests...", KERNEL_TESTS.len());
    let mut failed = 0;
    for test in KERNEL_TESTS {
        unsafe {
            CURRENT_TEST = Some(&test);
        }

        print!("\n");
        println!("running {}...", test.name);
        let passed = (test.test_fn)(boot_info);
        println!(
            "\t {}: [\x1b[{}m{}\x1b[0m]",
            test.name,
            if passed { "32" } else { "31" },
            if passed { "OK" } else { "FAILED" }
        );
        if !passed {
            failed += 1;
        }
    }
    if failed > 0 {
        println!("\n{} test(s) failed!", failed);
    } else {
        println!("all tests passed!");
    }

    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(test) = unsafe { CURRENT_TEST } {
        println!("\t {}: [\x1b[31mPANICED\x1b[0m]", test.name);
        println!(
            "test location: {}:{}:{}\n",
            test.location.file, test.location.line, test.location.column
        );
    }

    println!("detailed panic info:");

    kernel_panic(info);
}
