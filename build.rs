use std::path::PathBuf;

extern crate bootloader;
extern crate bootloader_boot_config;

use bootloader::BootConfig;
use bootloader_boot_config::LevelFilter;

fn main() {
    let mut config = BootConfig::default();
    // config.log_level = LevelFilter::Off;
    // config.frame_buffer_logging = false;

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let kernel =
        PathBuf::from(std::env::var_os("CARGO_BIN_FILE_MONOS_KERNEL_monos_kernel").unwrap());

    // create an UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .set_boot_config(&config)
        .create_disk_image(&uefi_path)
        .unwrap();

    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .set_boot_config(&config)
        .create_disk_image(&bios_path)
        .unwrap();

    // pass the disk image path/* s as env variables t */o the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
