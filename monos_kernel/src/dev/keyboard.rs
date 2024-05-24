use crate::acpi::{tables, ACPI_ROOT};
use crate::interrupts::{
    apic::{io_apic::DeliveryMode, LOCAL_APIC},
    InterruptStackFrame,
};

use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};
use spin::{Lazy, Mutex};
use x86_64::instructions::port::Port;

static KEYBOARD: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>>> = Lazy::new(|| {
    Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    ))
});

pub fn init() {
    let madt = ACPI_ROOT
        .get()
        .expect("ACPI not initialized yet")
        .get_table::<tables::MADT>()
        .expect("no MADT table found");

    let processor_local_apic = madt
        .get_entries::<tables::madt::ProcessorLocalAPIC>()
        .next()
        .expect("no processor local APIC found")
        .apic_id();

    let global_system_interrupt_val = madt
        .get_entries::<tables::madt::InterruptSourceOverride>()
        .find(|entry| entry.source() == 1)
        .map(|entry| entry.global_system_interrupt())
        .unwrap_or(1);

    let mut io_apic = madt
        .get_entries::<tables::madt::IOAPIC>()
        .next()
        .expect("no IO APIC found")
        .get_ioapic();

    let mut entry = io_apic.ioredtbl(global_system_interrupt_val);
    entry.set_vector(0x21);
    entry.set_delivery_mode(DeliveryMode::Fixed);
    entry.set_destination_mode(false);
    entry.set_pin_polarity(false);
    entry.set_trigger_mode(false);
    entry.set_masked(false);
    entry.set_destination(processor_local_apic);
    io_apic.set_ioredtbl(global_system_interrupt_val, entry);
}

pub extern "x86-interrupt" fn interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode('\n') => crate::gfx::framebuffer().confirm_input(),
                DecodedKey::Unicode('\u{8}') => crate::gfx::framebuffer().delete_input_char(),
                DecodedKey::Unicode(character) => {
                    crate::gfx::framebuffer().add_input_char(character)
                }
                _ => {}
            }
        }
    }

    LOCAL_APIC.get().unwrap().eoi();
}
