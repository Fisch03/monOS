use crate::LOWER_HALF_END;

use crate::fs::{fs, ArrayPath, FileHandle, Path, PathBuf};
use core::mem::MaybeUninit;
use monos_std::io::SeekMode;

pub fn sys_open(arg1: u64, arg2: u64, arg3: u64) {
    assert!(arg1 + arg2 < LOWER_HALF_END);
    assert!(arg3 + (size_of::<Option<FileHandle>>() as u64) < LOWER_HALF_END);

    let path = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            arg1 as *const u8,
            arg2 as usize,
        ))
        .expect("invalid utf8 string")
    };
    let path = Path::new(path);
    // crate::print!("SYS: sys_open: {:?}", path);

    let file_handle_ptr = arg3 as *mut Option<FileHandle>;
    let file_handle = unsafe { &mut *file_handle_ptr };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    *file_handle = current_proc.open(path);

    // if let Some(file_handle) = file_handle {
    //     crate::print!(" -> {:?}\n", file_handle);
    // } else {
    //     crate::print!(" -> failed!\n");
    // }
}

pub fn sys_close(arg1: u64) {
    let file_handle = FileHandle::new(arg1);
    // crate::println!("sys_close: {:?}", file_handle);

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();
    match current_proc.close(file_handle) {
        Ok(_) => (),
        Err(e) => crate::println!(
            "sys_close: failed to close file handle {:?}: {:?}",
            file_handle,
            e
        ),
    }
}

pub fn sys_seek(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let file_handle = FileHandle::new(arg1);
    let offset = arg2 as i64;
    let seek_mode = SeekMode::try_from(arg3 as u8).expect("invalid seek mode");

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();
    let pos = current_proc.seek(file_handle, offset, seek_mode) as u64;
    // crate::println!(
    //     "sys_seek: {:?}: {} from {:?} -> {}",
    //     FileHandle::new(arg1),
    //     offset,
    //     SeekMode::try_from(arg3 as u8).expect("invalid seek mode"),
    //     pos
    // );

    pos
}

pub fn sys_read(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    assert!(arg2 + arg3 < LOWER_HALF_END);

    let file_handle = FileHandle::new(arg1);
    let mut buf = unsafe { core::slice::from_raw_parts_mut(arg2 as *mut u8, arg3 as usize) };

    let mut current_proc = crate::process::CURRENT_PROCESS.write();
    let current_proc = current_proc.as_mut().unwrap();

    // crate::println!("sys_read: {} bytes from {:?}", buf.len(), file_handle);
    if let Some(read) = current_proc.read(file_handle, &mut buf) {
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

// arg1: ptr to path string
// arg2: length of path string
// arg3: ptr to slice of ArrayPaths
// arg4: amount of ArrayPath space in slice
//
// returns number of paths written to slice
pub fn sys_list(arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    assert!(arg1 + arg2 < LOWER_HALF_END);
    assert!(arg3 + (size_of::<ArrayPath>() as u64) * 4 < LOWER_HALF_END);

    let path = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            arg1 as *const u8,
            arg2 as usize,
        ))
        .expect("invalid utf8 string")
    };
    let path = PathBuf::from(path);
    // crate::println!("sys_list: {:?}", path);

    let paths = unsafe {
        core::slice::from_raw_parts_mut(arg3 as *mut MaybeUninit<ArrayPath>, arg4 as usize)
    };

    let mut i = 0;
    if let Some(parent) = fs().get(&path) {
        for node in parent.children().iter() {
            if i >= paths.len() {
                break;
            }
            let mut new_path = ArrayPath::new();
            new_path.push_str(path.as_str());
            new_path.push_str("/");
            new_path.push_str(node.name());

            paths[i] = MaybeUninit::new(new_path);
            i += 1;
        }
    }
    i as u64
}
