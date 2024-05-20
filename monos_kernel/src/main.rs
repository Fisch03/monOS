#![no_std]
#![no_main]

use bootloader_api::{config, BootInfo, BootloaderConfig};
use core::arch::asm;

use monos_kernel::*;

extern crate alloc;
use alloc::{boxed::Box, rc::Rc, vec::Vec};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(config::Mapping::Dynamic);

    config
};
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    monos_kernel::kernel_init(boot_info);

    println!("hello world!! :D\nthis is a new line");
    println!();

    interrupts::breakpoint();

    println!();

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    //always print to serial first. the screen might not be
    //initialized yet
    dbg!(info);

    // eprintln!("oh noes! the kernel {}", info);

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
