use super::SegmentSelector;
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
