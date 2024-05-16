mod pic;

mod local_apic;
use local_apic::LocalAPIC;

use crate::arch::registers::MSR;
use crate::mem::VirtualAddress;
use crate::utils::BitField;
use core::{fmt, ops::Range};

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
    const APIC_BASE_ADDRESS: Range<usize> = 12..52;

    pub fn read() -> Self {
        let reg = MSR::new(Self::IA32_APIC_BASE_MSR);

        // safety: apic_base is a valid MSR
        Self(unsafe { reg.read() })
    }

    pub fn write(&self) {
        let reg = MSR::new(Self::IA32_APIC_BASE_MSR);
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

    pub fn address(&self) -> u64 {
        self.0.get_bits(Self::APIC_BASE_ADDRESS)
    }

    pub fn set_address(&mut self, address: VirtualAddress) -> &mut Self {
        self.0.set_bits(Self::APIC_BASE_ADDRESS, address.as_u64());
        self
    }
}

impl fmt::Debug for APICBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("APICBase")
            .field("bootstrap_cpu", &self.is_bootstrap_cpu())
            .field("x2apic_mode", &self.x2apic_mode())
            .field("apic_enabled", &self.apic_enabled())
            .field("address", &format_args!("{:#x}", self.address()))
            .finish()
    }
}

static LOCAL_APIC: Once<LocalAPIC> = Once::new();

pub fn init() {
    //make sure the PIC doesn't get in the way
    pic::disable_pic();

    //TODO (?) disable PIC mode

    let mut apic_base = APICBase::read();
    apic_base.set_apic_enabled(true);
    apic_base.set_x2apic_mode(false);

    // TODO: map a page for the APIC
    // LOCAL_APIC.call_once(|| LocalAPIC::new(virtual_address));

    apic_base.write();
}
