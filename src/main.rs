use clap::Parser;
use std::process;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    test: bool,

    #[arg(short, long)]
    debug: bool,

    #[arg(short = 'i', long)]
    show_interrupts: bool,

    #[arg(long)]
    ui: bool,

    #[arg(long)]
    qemu_term: bool,
}

fn main() {
    let args = Args::parse();

    // read env variables that were set in build script
    let (uefi_path, kernel_path) = if !args.test {
        (
            env!("UEFI_PATH_MONOS_KERNEL"),
            env!("KERNEL_PATH_MONOS_KERNEL"),
        )
    } else {
        (
            env!("UEFI_PATH_TEST_KERNEL"),
            env!("KERNEL_PATH_TEST_KERNEL"),
        )
    };

    let mut child_ids = vec![];
    let mut main_proc = qemu_run(uefi_path, &args);
    child_ids.push(main_proc.id());

    if args.debug {
        ctrlc::set_handler(|| {}).unwrap(); // let gdb handle the interrupt

        main_proc = gdb_run(kernel_path);
        child_ids.push(main_proc.id());
    }

    main_proc.wait().unwrap();

    for id in child_ids {
        unsafe {
            libc::kill(id as i32, libc::SIGKILL);
        }
    }
}

fn qemu_run(uefi_path: &str, args: &Args) -> process::Child {
    let mut cmd = if !args.qemu_term {
        process::Command::new("qemu-system-x86_64")
    } else {
        let mut cmd = process::Command::new("kitty");
        cmd.arg("--").arg("qemu-system-x86_64");
        cmd
    };

    cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());

    if args.debug {
        cmd.arg("-s").arg("-S");

        // let gdb be the output
        // cmd.stdout(process::Stdio::null());
        // cmd.stderr(process::Stdio::null());
        cmd.stdin(process::Stdio::null());

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    // detach from the parent process group to ignore ctrl+c
                    libc::setsid();
                    Ok(())
                });
            }
        }
    }

    if args.show_interrupts {
        cmd.arg("-d").arg("int");
    }

    if !args.ui {
        cmd.arg("-display").arg("sdl"); // sdl handles scaled display a lot better
    }
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-m").arg("512M");
    cmd.arg("-drive")
        .arg(format!("format=raw,file={uefi_path}"));

    cmd.spawn().unwrap()
}

fn gdb_run(bin_path: &str) -> process::Child {
    let mut cmd = process::Command::new("gdb");
    cmd.arg("-tui");
    cmd.arg("-ex").arg("target remote:1234");
    cmd.arg("-ex")
        .arg(format!("symbol-file {} -o 0xffff800000000000", bin_path));

    cmd.spawn().unwrap()
}
