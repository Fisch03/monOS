use crate::LOWER_HALF_END;

use crate::fs::{fs, DirEntry, DirIter};
use crate::process::Process;
use monos_std::filesystem::{FileHandle, Path};

use alloc::boxed::Box;

pub fn sys_open(arg1: u64, arg2: u64, arg3: u64) {
    assert!(arg1 < LOWER_HALF_END);
    assert!(arg1 + arg2 < LOWER_HALF_END);
    assert!(arg3 < LOWER_HALF_END);

    let path = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            arg1 as *const u8,
            arg2 as usize,
        ))
        .expect("invalid utf8 string")
    };
    let path = Path::new(path);

    let file_handle_ptr = arg3 as *mut Option<FileHandle>;
    let file_handle = unsafe { &mut *file_handle_ptr };

    if let Ok(Some(file)) = fs().iter_root_dir().get_entry(path).map(|f| f.as_file()) {
        let mut current_proc = crate::process::CURRENT_PROCESS.write();
        let current_proc = current_proc.as_mut().unwrap();

        *file_handle = Some(current_proc.open_file(Box::new(file)));
    } else {
        crate::println!("sys_open: failed to open file");

        *file_handle = None;
    }
}

pub fn sys_read(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    assert!(arg2 < LOWER_HALF_END);
    assert!(arg2 + arg3 < LOWER_HALF_END);

    let mut buf = unsafe { core::slice::from_raw_parts_mut(arg2 as *mut u8, arg3 as usize) };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    if let Some(read) = current_proc.read_file(FileHandle::new(arg1), &mut buf) {
        return read as u64;
    } else {
        crate::println!(
            "sys_read: process {:?} tried to read from invalid file handle {}",
            current_proc.id(),
            arg1
        );

        return 0;
    }
}
