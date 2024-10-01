use crate::{fs::Path, process, LOWER_HALF_END};

pub fn sys_spawn(arg1: u64, arg2: u64) -> u64 {
    assert!(arg1 + arg2 < LOWER_HALF_END);

    let path = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            arg1 as *const u8,
            arg2 as usize,
        ))
        .expect("invalid utf8 string")
    };
    let path = Path::new(path);
    crate::println!("sys_spawn: {:?}", path);

    let result = process::spawn(path);

    match result {
        Ok(pid) => pid.as_u32() as u64,
        Err(e) => {
            crate::println!("spawn failed: {:?}", e);
            0
        }
    }
}
