use crate::arch::registers::MSR;
use crate::gdt;
use crate::mem::VirtualAddress;

use monos_std::syscall::{Syscall, SyscallType};

use crate::process::Context;
use core::arch::asm;

mod fs;
mod ipc;
mod process;

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
            // get access to kernel stack
            "swapgs",
            "mov gs:{temp_stack}, rsp", // save current rsp

            "mov rsp, gs:{kernel_stack}",
            "sub rsp, {kernel_stack_offset}",

            "sub rsp, 8",
            "push gs:{temp_stack}",
            "swapgs", // switch to user gs

            // backup registers for sysretq
            "push r11",  // rflags
            "sub rsp, 8", // will be filled with CS
            "push rcx",  // rip

            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",

            "push rdi",
            "push rsi",
            "push rbp",
            "push r8",

            "push r9",
            "push r10",
            "push r11",
            "push r12",

            "push r13",
            "push r14",
            "push r15",

            // convert syscall args to c abi
            // c abi:   rdi, rsi, rdx, rcx, r8, r9
            // syscall: rax, rdi, rsi, rdx, r10
            "mov r9, rsp", // context ptr
            "mov r8, r10",
            "mov rcx, rdx",
            "mov rdx, rsi",
            "mov rsi, rdi",
            "mov rdi, rax",

            // call the rust handler
            "call {dispatch_syscall}",

            "pop r15", // restore callee-saved registers
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
            "add rsp, 8", // pop rax without restoring it (it contains the return value now)

            "add rsp, 24", // Skip RIP, CS and RFLAGS
            "pop rsp", // Restore user stack

            "sysretq", // back to userland

            kernel_stack = const(0x24 + gdt::TIMER_IST_INDEX * 8),
            temp_stack = const(0x24 + gdt::SYSCALL_TEMP_INDEX * 8),
            kernel_stack_offset = const(1024),
            dispatch_syscall = sym dispatch_syscall,
            options(noreturn));
    }
}

extern "C" fn dispatch_syscall(
    syscall_id: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    context_addr: u64,
) -> u64 {
    // crate::println!("syscall {} {} {} {} {}", syscall_id, arg1, arg2, arg3, arg4);
    let context_addr = VirtualAddress::new(context_addr);

    // fix the context cs and ss
    unsafe {
        let context: &mut Context = &mut *context_addr.as_mut_ptr();
        context.cs = gdt::GDT.1.user_code.as_u16() as u64;
        context.ss = gdt::GDT.1.user_data.as_u16() as u64;
    }

    let mut ret = 0;
    if let Ok(syscall) = Syscall::try_from(syscall_id) {
        match syscall.ty {
            SyscallType::Spawn => ret = process::sys_spawn(arg1, arg2),
            SyscallType::Yield => process::sys_yield(context_addr),

            SyscallType::Serve => ipc::sys_serve(arg1, arg2, arg3),
            SyscallType::Connect => ipc::sys_connect(arg1, arg2, arg3),
            SyscallType::WaitConnect => panic!("unimplemented syscall {:?}", syscall),
            SyscallType::Receive => ipc::sys_receive(syscall.get_handle(), arg1),
            SyscallType::ReceiveAny => ipc::sys_receive_any(arg1),
            SyscallType::Send => ipc::sys_send(
                syscall.get_handle(),
                syscall.is_chunk(),
                arg1,
                arg2,
                arg3,
                arg4,
            ),
            SyscallType::SendSync => panic!("unimplemented syscall {:?}", syscall),

            SyscallType::RequestChunk => ret = ipc::sys_request_chunk(arg1),

            SyscallType::Open => fs::sys_open(arg1, arg2, arg3),
            SyscallType::Seek => panic!("unimplemented syscall {:?}", syscall),
            SyscallType::Read => ret = fs::sys_read(arg1, arg2, arg3),
            SyscallType::Write => panic!("unimplemented syscall {:?}", syscall),

            SyscallType::List => ret = fs::sys_list(arg1, arg2, arg3, arg4),

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

    ret
}
