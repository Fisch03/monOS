use std::fs;
use std::path::{Path, PathBuf};

extern crate bootloader;
extern crate bootloader_boot_config;
extern crate fatfs;
extern crate glob;

use build_print::info;
use glob::glob;

use bootloader::BootConfig;
#[allow(unused_imports)]
use bootloader_boot_config::LevelFilter;

const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const DISK_SIZE_PAD: u64 = 3 * MB; // ~size that the size that the user gets in the ramdisk

#[derive(Debug)]
struct KernelOptions {
    bin_glob: Option<String>,
}

impl Default for KernelOptions {
    fn default() -> Self {
        KernelOptions { bin_glob: None }
    }
}

fn main() {
    // info!("updating submodules");
    // std::process::Command::new("git")
    //     .args(&["submodule", "update", "--init", "--recursive"])
    //     .status()
    //     .unwrap();

    make_kernel(
        "test_kernel",
        KernelOptions {
            bin_glob: Some(String::from("test_*")),
        },
    );
    make_kernel("monos_kernel", KernelOptions::default());
}

fn make_kernel(dependency: &str, options: KernelOptions) {
    info!("building kernel {}", dependency);

    #[allow(unused_mut)]
    let mut config = BootConfig::default();
    // config.log_level = LevelFilter::Off;
    config.frame_buffer_logging = false;
    config.frame_buffer.wanted_framebuffer_width = Some(640);
    config.frame_buffer.wanted_framebuffer_height = Some(480);

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join(dependency);
    let kernel = PathBuf::from(
        std::env::var_os(format!("CARGO_BIN_FILE_MONOS_KERNEL_{}", dependency)).unwrap(),
    );

    println!(
        "cargo:rustc-env=KERNEL_PATH_{}={}",
        dependency.to_uppercase(),
        kernel.display()
    );

    let os_disk_in = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("os_disk");
    let userspace_prog_dir =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("userspace");

    println!("cargo:rerun-if-changed={}", os_disk_in.display());
    let os_disk_out = out_dir.join("os_disk");
    copy_dir_all(&os_disk_in, &os_disk_out).unwrap();
    build_userspace(&userspace_prog_dir, &os_disk_out.join("bin"), &options);
    let ramdisk_path = build_disk(&os_disk_out, &out_dir);

    info!("  └> creating UEFI disk image");
    let uefi_path = out_dir.join(format!("{}_uefi.img", dependency));

    bootloader::UefiBoot::new(&kernel)
        .set_boot_config(&config)
        .set_ramdisk(&ramdisk_path)
        .create_disk_image(&uefi_path)
        .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!(
        "cargo:rustc-env=UEFI_PATH_{}={}",
        dependency.to_uppercase(),
        uefi_path.display()
    );

    info!("  └> done! path: '{}'", uefi_path.display());
}

fn build_userspace(crates_dir: &Path, out_dir: &Path, options: &KernelOptions) {
    info!("  └> building userspace");
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("target"))
        .join("userspace");

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("x86_64-monos_user.json").display()
    );

    let crates_glob = options.bin_glob.as_ref().map_or("*", |g| g.as_str());
    let crates_dir = crates_dir.join(crates_glob);

    for user_crate in glob(&crates_dir.to_str().unwrap()).unwrap() {
        let user_crate = user_crate.unwrap();
        println!("cargo:rerun-if-changed={}", user_crate.display());
        let crate_name = String::from(user_crate.file_name().unwrap().to_string_lossy());

        info!("      └> building bin '{}'", crate_name);

        let mut cargo = std::process::Command::new("cargo");
        cargo
            .arg("rustc")
            .arg("--release")
            .arg("--bin")
            .arg(&crate_name)
            .arg("--target")
            .arg(manifest_dir.join("x86_64-monos_user.json"))
            .arg("--manifest-path")
            .arg(user_crate.join("Cargo.toml"))
            .arg("-Zbuild-std=core,alloc,compiler_builtins")
            .arg("-Zbuild-std-features=compiler-builtins-mem")
            .arg("--")
            .arg("-Clink-arg=--image-base=0x200000")
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
    info!("  └> building disk image");
    println!("cargo:rerun-if-changed={}", in_dir.display());

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
    info!("      └> disk size: {} KB", fat_size / KB);

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

        #[cfg(target_os = "windows")]
        let relative_path = relative_path.replace("\\", "/");
        if full_path.is_dir() {
            match fs_root.create_dir(&relative_path) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Error creating directory {:?}: {:?}", relative_path, e);
                }
            }
        } else {
            let mut file = fs_root.create_file(&relative_path).unwrap();
            let mut source = fs::File::open(&full_path).unwrap();
            std::io::copy(&mut source, &mut file).unwrap();
        }
    }

    disk_image_path
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::remove_dir_all(&dst).ok();
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
