use std::fs;
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

    let userspace_prog_dir =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("userspace");

    build_userspace(&userspace_prog_dir, &os_disk_in_dir.join("bin"));
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

fn build_userspace(crates_dir: &Path, out_dir: &Path) {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("target"));

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("monos_std").display()
    );

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("x86_64-monos_user.json").display()
    );

    for user_crate in fs::read_dir(crates_dir).unwrap() {
        println!(
            "cargo:rerun-if-changed={}",
            user_crate.as_ref().unwrap().path().display()
        );
        let user_crate = user_crate.unwrap();
        let crate_name = user_crate.file_name().into_string().unwrap();
        let crate_path = user_crate.path();
        let mut cargo = std::process::Command::new("cargo");
        cargo
            .arg("rustc")
            .arg("--release")
            .arg("--target")
            .arg(manifest_dir.join("x86_64-monos_user.json"))
            .arg("-Zbuild-std=core,alloc,compiler_builtins")
            .arg("-Zbuild-std-features=compiler-builtins-mem")
            .arg("--manifest-path")
            .arg(crate_path.join("Cargo.toml"))
            .arg("--")
            .arg("-Clink-arg=--image-base=0x10000")
            .env("CARGO_TARGET_DIR", &target_dir);

        dbg!(&cargo);

        let status = cargo.status().unwrap();
        assert!(status.success());

        let bin_file = target_dir
            .join("x86_64-monos_user")
            .join("release")
            .join(&crate_name);

        assert!(bin_file.exists());

        fs::copy(&bin_file, out_dir.join(&crate_name)).unwrap();
    }
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
        fatfs::FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat16)
            .fats(1),
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
        if full_path.ends_with(".gitkeep") {
            continue;
        }
        let relative_path = full_path.strip_prefix(&in_dir).unwrap().to_str().unwrap();

        if full_path.is_dir() {
            fs_root.create_dir(&relative_path).unwrap();
        } else {
            #[cfg(target_os = "windows")]
            let relative_path = relative_path.replace("\\", "/");
            let mut file = fs_root.create_file(&relative_path).unwrap();
            let mut source = fs::File::open(&full_path).unwrap();
            std::io::copy(&mut source, &mut file).unwrap();
        }
    }

    disk_image_path
}
