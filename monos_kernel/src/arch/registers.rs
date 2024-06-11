use crate::gdt::SegmentSelector;
use crate::mem::{Frame, PageSize4K, PhysicalAddress};

use core::arch::asm;

pub unsafe fn set_cs(selector: SegmentSelector) {
    asm!(
        "push {sel}",
        "lea {tmp}, [1f + rip]",
        "push {tmp}",
        "retfq",
        "1:",
        sel = in(reg) u64::from(selector.as_u16()),
        tmp = lateout(reg) _,
        options(preserves_flags)
    );
}

macro_rules! set_segment {
    ($segment:literal, $selector:expr) => {
        unsafe {
            asm!(
                concat!("mov ", $segment, ", {0:x}"),
                in(reg) $selector.as_u16(),
                options(nostack, preserves_flags)
            );
        }
    };
}

pub unsafe fn set_ds(selector: SegmentSelector) {
    set_segment!("ds", selector);
}

pub unsafe fn set_es(selector: SegmentSelector) {
    set_segment!("es", selector);
}

pub unsafe fn set_ss(selector: SegmentSelector) {
    set_segment!("ss", selector);
}

pub struct CR3;
impl CR3 {
    #[inline]
    pub fn read() -> (Frame<PageSize4K>, u16) {
        let value: u64;

        unsafe {
            asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        }

        let addr = PhysicalAddress::new(value & 0x0000_ffff_ffff_f000);
        let frame = Frame::around(addr);

        (frame, (value & 0xFFF) as u16)
    }

    #[inline]
    pub unsafe fn write(frame: Frame<PageSize4K>, flags: u16) {
        let value = frame.start_address().as_u64() | u64::from(flags);
        unsafe {
            asm!("mov cr3, {}", in(reg) value, options(nomem, nostack));
        }
    }
}

pub struct CR4;
impl CR4 {
    pub const ENABLE_MACHINE_CHECK: usize = 6;
    pub const ENABLE_SSE: usize = 9;
    pub const ENABLE_UNMASKED_SSE: usize = 10;
    pub const TIME_STAMP_DISABLE: usize = 2;

    #[inline]
    pub fn read() -> u64 {
        let value: u64;
        unsafe {
            asm!("mov {}, cr4", out(reg) value, options(nomem, nostack, preserves_flags));
        }
        value
    }
    #[inline]
    pub unsafe fn write(value: u64) {
        unsafe {
            asm!("mov cr4, {}", in(reg) value, options(nomem, nostack));
        }
    }
}

pub struct MSR(u32);
impl MSR {
    #[inline]
    pub const fn new(reg: u32) -> Self {
        MSR(reg)
    }

    #[inline]
    pub unsafe fn read(&self) -> u64 {
        let low: u32;
        let high: u32;

        unsafe {
            asm!(
                "rdmsr",
                in("ecx") self.0,
                out("eax") low,
                out("edx") high,
                options(nomem, nostack, preserves_flags)
            );
        }

        ((high as u64) << 32) | (low as u64)
    }

    #[inline]
    pub unsafe fn write(&mut self, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;

        unsafe {
            asm!(
                "wrmsr",
                in("ecx") self.0,
                in("eax") low,
                in("edx") high
            );
        }
    }
}
