use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    test: bool,
}

fn main() {
    let args = Args::parse();

    // read env variables that were set in build script
    let kernel_uefi_path = env!("UEFI_PATH_MONOS_KERNEL");
    let test_uefi_path = env!("UEFI_PATH_TEST_KERNEL");

    if !args.test {
        qemu_run(kernel_uefi_path);
    } else {
        qemu_run(test_uefi_path);
    }
}

fn qemu_run(uefi_path: &str) {
    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    // cmd.arg("-s").arg("-S");
    cmd.arg("-display").arg("sdl"); // sdl handles scaled display a lot better
    cmd.arg("-serial").arg("stdio");
    // cmd.arg("-m").arg("512M");
    cmd.arg("-drive")
        .arg(format!("format=raw,file={uefi_path}"));
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
