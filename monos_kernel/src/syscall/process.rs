use crate::{
    fs::Path,
    mem::VirtualAddress,
    process::{self, Context},
    LOWER_HALF_END,
};
use core::arch::asm;

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

pub fn sys_yield(current_context_addr: VirtualAddress) {
    let context_addr = process::schedule_next(current_context_addr);

    if context_addr.as_u64() == 0 {
        return;
    }

    unsafe {
        asm!(
        "mov rsp, rdi", // Set the stack to the Context address

        // Pop scratch registers from new stack
        "pop r15",
        "pop r14",
        "pop r13",

        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",

        "pop r8",
        "pop rbp",
        "pop rsi",
        "pop rdi",

        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",

        "sti",

        "iretq",
        in("rdi") context_addr.as_u64(),
        options(noreturn))
    }
}
