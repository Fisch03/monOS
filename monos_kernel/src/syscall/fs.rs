use crate::LOWER_HALF_END;

use monos_std::filesystem::*;

pub fn sys_open(arg1: u64, arg2: u64, arg3: u64) {
    assert!(arg1 < LOWER_HALF_END);
    assert!(arg1 + arg2 < LOWER_HALF_END);

    let path = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            arg1 as *const u8,
            arg2 as usize,
        ))
        .expect("invalid utf8 string")
    };
    let path = Path::new(path);

    todo!();
}

pub fn sys_read(arg1: u64, arg2: u64, arg3: u64) {
    assert!(arg2 < LOWER_HALF_END);
    assert!(arg2 + arg3 < LOWER_HALF_END);

    let buf = unsafe { core::slice::from_raw_parts_mut(arg2 as *mut u8, arg3 as usize) };

    todo!();
}
