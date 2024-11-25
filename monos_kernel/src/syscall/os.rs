pub fn print(arg1: u64, arg2: u64) {
    assert!(arg1 + arg2 < crate::LOWER_HALF_END);

    let s = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(
            arg1 as *const u8,
            arg2 as usize,
        ))
        .expect("invalid utf8 string")
    };

    crate::print!("{}", s);
}

pub fn get_system_time() -> u64 {
    crate::dev::HPET.boot_time_ms()
}
