use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

extern crate bootloader;
extern crate bootloader_boot_config;
extern crate fatfs;
extern crate glob;

use glob::glob;

use bootloader::BootConfig;
#[allow(unused_imports)]
use bootloader_boot_config::LevelFilter;

const MB: u64 = 1024 * 1024;
const DISK_SIZE_PAD: u64 = 3 * MB; // ~size that the size that the user gets in the ramdisk

fn main() {
    #[allow(unused_mut)]
    let mut config = BootConfig::default();
    // config.log_level = LevelFilter::Off;
    config.frame_buffer_logging = false;

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let kernel =
        PathBuf::from(std::env::var_os("CARGO_BIN_FILE_MONOS_KERNEL_monos_kernel").unwrap());

    let os_disk_in_dir =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("os_disk");
    println!("cargo:rerun-if-changed={}", os_disk_in_dir.display());

    let ramdisk_img_path = build_disk(&os_disk_in_dir, &out_dir);

    // create an UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .set_boot_config(&config)
        .set_ramdisk(&ramdisk_img_path)
        .create_disk_image(&uefi_path)
        .unwrap();

    // create a BIOS disk image
    // let bios_path = out_dir.join("bios.img");
    // bootloader::BiosBoot::new(&kernel)
    //     .set_boot_config(&config)
    //     .create_disk_image(&bios_path)
    //     .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    // println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}

fn build_disk(in_dir: &Path, out_dir: &Path) -> PathBuf {
    let disk_image_path = out_dir.join("disk.img");
    let _ = fs::remove_file(&disk_image_path);

    let mut disk_image = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&disk_image_path)
        .unwrap();

    let file_paths = in_dir.join("**/*").to_str().unwrap().to_string();
    let mut file_size = 0;
    for entry in glob(&file_paths).unwrap() {
        let full_path = entry.unwrap().to_owned();
        file_size += full_path.metadata().unwrap().len();
    }

    const MB: u64 = 1024 * 1024;
    let fat_size = ((file_size) / MB + 1) * MB + MB;
    disk_image.set_len(fat_size + DISK_SIZE_PAD).unwrap();

    fatfs::format_volume(
        &mut disk_image,
        fatfs::FormatVolumeOptions::new().fat_type(fatfs::FatType::Fat16),
    )
    .expect("format failed");

    let fs = fatfs::FileSystem::new(disk_image, fatfs::FsOptions::new()).expect("fs failed");
    assert!(
        fs.fat_type() == fatfs::FatType::Fat16,
        "disk isn't big enough for FAT16"
    );
    let fs_root = fs.root_dir();

    for entry in glob(&file_paths).unwrap() {
        let full_path = entry.unwrap().to_owned();
        let relative_path = full_path.strip_prefix(&in_dir).unwrap().to_str().unwrap();

        if full_path.is_dir() {
            fs_root.create_dir(&relative_path).unwrap();
        } else {
            let mut file = fs_root.create_file(&relative_path).unwrap();
            let mut source = fs::File::open(&full_path).unwrap();
            std::io::copy(&mut source, &mut file).unwrap();
        }
    }

    disk_image_path
}
