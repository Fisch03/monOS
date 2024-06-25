use crate::arch::registers::MSR;
use crate::gdt;

use monos_std::syscall::{Syscall, SyscallType};

use core::arch::asm;

mod io;

const IA32_EFER_MSR: u32 = 0xC0000080;
const IA32_STAR_MSR: u32 = 0xC0000081;
const IA32_LSTAR_MSR: u32 = 0xC0000082;
const IA32_FMASK_MSR: u32 = 0xC0000084;

pub fn init() {
    // disable interrupts during syscall
    let mut ia32_fmask = MSR::new(IA32_FMASK_MSR);
    unsafe { ia32_fmask.write(0x200) };

    // set syscall handler
    let handler_addr = handle_syscall as *const () as u64;
    let mut ia32_lstar = MSR::new(IA32_LSTAR_MSR);
    unsafe { ia32_lstar.write(handler_addr) };

    // TODO: make this a bit cleaner
    // kernel code segment selector: 0x8
    // user data/code segment selector: 0x20
    // ring 3
    let mut ia32_star = MSR::new(IA32_STAR_MSR);
    unsafe { ia32_star.write(0x23000800000000) };

    // enable syscall/sysret
    let mut ia32_efer = MSR::new(IA32_EFER_MSR);
    unsafe { ia32_efer.write(ia32_efer.read() | 1) };
}

#[no_mangle]
#[naked]
extern "C" fn handle_syscall() {
    unsafe {
        asm!(
            // backup registers for sysretq
            "push rcx",
            "push r11",
            "push rbp",

            // save callee-saved registers
            "push rbx", 
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            // save syscall args
            "push r10",
            "push rdx",
            "push rsi",
            "push rdi",
            "push rax",
            
            // get access to kernel stack
            "swapgs",
            "mov rcx, rsp", // back up current rsp
            "mov rsp, gs:{kernel_stack}",
            "push rcx",

            // convert syscall args to c abi
            // c abi:   rdi, rsi, rdx, rcx, r8
            // syscall: rax, rdi, rsi, rdx, r10
            "mov r8, r10",
            "mov rcx, rdx",
            "mov rdx, rsi",
            "mov rsi, rdi",
            "mov rdi, rax",

            // call the rust handler
            "call {dispatch_syscall}",

            
            // switch back to original GS
            "pop rcx",
            "mov rsp, rcx", // restore original rsp
            "swapgs",

            // restore syscall args
            "pop rax",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop r10",

            // restore callee-saved registers
            "pop r15", 
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

             // restore stack and registers for sysretq
            "pop rbp",
            "pop r11",
            "pop rcx",
            "sysretq", // back to userland

            kernel_stack = const(0x24 + gdt::TIMER_IST_INDEX * 8),
            dispatch_syscall= sym dispatch_syscall,
            options(noreturn));
    }
}

extern "C" fn dispatch_syscall(syscall_id: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) {
    if let Ok(syscall) = Syscall::try_from(syscall_id) {
        match syscall.ty {
            SyscallType::Print => {
                assert!(arg1 < crate::LOWER_HALF_END);
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
            SyscallType::Connect => io::sys_connect(arg1, arg2, arg3),
            SyscallType::Receive => io::sys_receive(syscall.get_handle(), arg1),
            SyscallType::ReceiveAny => io::sys_receive_any(arg1),
            SyscallType::Send => io::sys_send(syscall.get_handle(), arg1, arg2, arg3, arg4),
            _ => crate::println!("unimplemented syscall {:?}", syscall),
        }
    } else {
        crate::println!(
            "unknown syscall {} {} {} {} {}",
            syscall_id,
            arg1,
            arg2,
            arg3,
            arg4
        );
    }
}
