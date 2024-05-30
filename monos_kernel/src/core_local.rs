use crate::arch::registers::MSR;
use crate::gdt::TaskStateSegment;
use crate::mem::VirtualAddress;

use core::sync::atomic::{AtomicUsize, Ordering};
use core::{arch::asm, cell::Cell, mem, ptr};

static CPUS_ONLINE: AtomicUsize = AtomicUsize::new(0);

const GS_BASE: u32 = 0xC000_0101;

pub struct CoreLocal {
    this: *const Self,

    core_id: u32,

    pub tss: Cell<*mut TaskStateSegment>,
    pub kernel_stack: Cell<*mut u8>, // kernel stack of the currently running process
}

impl CoreLocal {
    pub fn init() {
        let core_id = CPUS_ONLINE.fetch_add(1, Ordering::Relaxed) as u32;
        let core_local = Self {
            this: ptr::null_mut(),
            core_id,
            kernel_stack: Cell::new(ptr::null_mut()),
            tss: Cell::new(ptr::null_mut()),
        };

        let core_local = if core_id == 0 {
            take_static::take_static! {
                static FIRST_CORE_LOCAL: Option<CoreLocal> = None;
            }
            FIRST_CORE_LOCAL.take().unwrap().insert(core_local)
        } else {
            todo!("multi-core support")
        };
        core_local.this = &*core_local;

        let mut gs_base = MSR::new(GS_BASE);
        unsafe { gs_base.write(VirtualAddress::from_ptr(&core_local).as_u64()) };
    }

    #[inline]
    pub fn get() -> &'static Self {
        unsafe {
            let raw: *const Self;
            asm!("mov {}, gs:{}", out(reg) raw, const mem::offset_of!(Self, this), options(nomem, nostack, preserves_flags));
            &*raw
        }
    }

    #[inline]
    pub fn core_id(&self) -> u32 {
        self.core_id
    }
}
