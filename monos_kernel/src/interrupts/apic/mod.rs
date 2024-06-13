mod pic;

mod local_apic;
use local_apic::{LocalAPIC, LocalAPICField};

pub mod io_apic;
pub use io_apic::IOAPIC;

use crate::arch::registers::MSR;
use crate::mem;
use crate::mem::PhysicalAddress;
use crate::utils::BitField;
use core::fmt;

use spin::Once;

/// APIC Base Address Register
/// ┌──┬──────────────┐
/// │ 0│              │
/// │  │   Reserved   │
/// │ 7│              │
/// ├──┼──────────────┤
/// │ 8│Boot Strap CPU│
/// ├──┼──────────────┤
/// │ 9│   Reserved   │
/// ├──┼──────────────┤
/// │10│ x2APIC Mode  │
/// ├──┼──────────────┤
/// │11│ APIC Enable  │
/// ├──┼──────────────┤
/// │12│              │
/// │  │              │
/// │  │  APIC Base   │
/// │  │   Address    │
/// │  │              │
/// │51│              │
/// ├──┼──────────────┤
/// │52│              │
/// │  │   Reserved   │
/// │63│              │
/// └──┴──────────────┘
struct APICBase(u64);
impl APICBase {
    const IA32_APIC_BASE_MSR: u32 = 0x1B;

    const BOOTSTRAP_CPU: usize = 8;
    const X2APIC_MODE: usize = 10;
    const APIC_ENABLE: usize = 11;

    pub fn read() -> Self {
        let reg = MSR::new(Self::IA32_APIC_BASE_MSR);

        // safety: apic_base is a valid MSR
        Self(unsafe { reg.read() })
    }

    pub fn write(&mut self) {
        let mut reg = MSR::new(Self::IA32_APIC_BASE_MSR);
        // safety: apic_base is a valid MSR
        unsafe { reg.write(self.0) }
    }

    pub fn is_bootstrap_cpu(&self) -> bool {
        self.0.get_bit(Self::BOOTSTRAP_CPU)
    }

    pub fn x2apic_mode(&self) -> bool {
        self.0.get_bit(Self::X2APIC_MODE)
    }

    pub fn set_x2apic_mode(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(Self::X2APIC_MODE, value);
        self
    }

    pub fn apic_enabled(&self) -> bool {
        self.0.get_bit(Self::APIC_ENABLE)
    }

    pub fn set_apic_enabled(&mut self, value: bool) -> &mut Self {
        self.0.set_bit(Self::APIC_ENABLE, value);
        self
    }

    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.0 & 0xFFFF_0000)
    }
}

impl fmt::Debug for APICBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("APICBase")
            .field("bootstrap_cpu", &self.is_bootstrap_cpu())
            .field("x2apic_mode", &self.x2apic_mode())
            .field("apic_enabled", &self.apic_enabled())
            .field("address", &self.address())
            .finish()
    }
}

pub static LOCAL_APIC: Once<LocalAPIC> = Once::new();

pub fn init() {
    //make sure the PIC doesn't get in the way
    pic::disable_pic();

    //TODO (?) disable PIC mode

    let mut apic_base = APICBase::read();
    apic_base.set_apic_enabled(true);
    apic_base.set_x2apic_mode(false);

    let frame = mem::Frame::around(apic_base.address());
    let page = mem::Page::around(mem::alloc_vmem(4096).align_up(4096));

    use mem::PageTableFlags;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::CACHE_DISABLE;

    unsafe { mem::map_to(&page, &frame, flags) }.expect("failed to map apic memory");

    LOCAL_APIC.call_once(|| {
        let mut local_apic = LocalAPIC::new(page.start_address());

        local_apic.write(LocalAPICField::TimerDivideConfig, 0b11);
        local_apic.write(LocalAPICField::TimerInitialCount, 100_000);

        local_apic
    });

    apic_base.write();
}
