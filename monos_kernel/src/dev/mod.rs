pub mod keyboard;
pub mod mouse;

use crate::acpi::{tables, ACPI_ROOT};

pub fn init() {
    let madt = ACPI_ROOT
        .get()
        .expect("ACPI not initialized yet")
        .get_table::<tables::MADT>()
        .expect("no MADT table found");

    let mut io_apic = madt
        .get_entries::<tables::madt::IOAPIC>()
        .next()
        .expect("no IO APIC found")
        .get_ioapic();

    keyboard::init(&madt, &mut io_apic);
    mouse::init(&madt, &mut io_apic);
}
