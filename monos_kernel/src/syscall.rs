use crate::arch::registers::MSR;

const IA32_EFER_MSR: u32 = 0xC0000080;
const IA32_STAR_MSR: u32 = 0xC0000081;

pub fn init() {
    let mut ia32_star = MSR::new(IA32_STAR_MSR);

    // TODO: make this a bit cleaner
    // kernel code segment selector: 0x8
    // user data/code segment selector: 0x20
    // ring 3
    unsafe { ia32_star.write(0x23000800000000) };

    let mut ia32_efer = MSR::new(IA32_EFER_MSR);
    // enable syscall/sysret
    unsafe { ia32_efer.write(ia32_efer.read() | 1) };
}
