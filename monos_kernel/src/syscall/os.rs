use monos_std::syscall::SysInfo;

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

pub fn sys_info(arg1: u64) -> u64 {
    match SysInfo::try_from(arg1) {
        Ok(SysInfo::SystemTime) => crate::dev::HPET.boot_time_ms(),

        Ok(SysInfo::FreeMemory) => crate::mem::free_memory(),
        Ok(SysInfo::UsedMemory) => crate::mem::used_memory(),
        Ok(SysInfo::TotalMemory) => crate::mem::total_memory(),

        Ok(SysInfo::ProcessId) => crate::process::CURRENT_PROCESS
            .read()
            .as_ref()
            .unwrap()
            .id()
            .as_u32() as u64,
        Ok(SysInfo::NumProcesses) => crate::process::num_processes() as u64,

        Err(_) => 0,
    }
}
