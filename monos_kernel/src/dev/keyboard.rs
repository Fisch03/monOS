use crate::acpi::tables;
use crate::interrupts::{
    apic::{
        io_apic::{DeliveryMode, IOAPIC},
        LOCAL_APIC,
    },
    InterruptIndex, InterruptStackFrame,
};
use crate::mem::Mapping;
use crate::process::messaging::{add_system_port, PartialSendChannelHandle, SYS_PORT_NO_RECEIVE};

use alloc::vec::Vec;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::{Lazy, Mutex, Once};
use x86_64::instructions::port::Port;

static KEYBOARD: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>>> = Lazy::new(|| {
    Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    ))
});

static LISTENERS: Lazy<Mutex<Vec<PartialSendChannelHandle>>> = Lazy::new(|| Mutex::new(Vec::new()));
static CHANNEL_HANDLE: Once<PartialSendChannelHandle> = Once::new();

pub fn add_listener(handle: PartialSendChannelHandle) -> PartialSendChannelHandle {
    LISTENERS.lock().push(handle);
    CHANNEL_HANDLE
        .get()
        .expect("keyboard channel not initialized")
        .clone()
}

pub fn init(madt: &Mapping<tables::MADT>, io_apic: &mut Mapping<IOAPIC>) {
    let global_system_interrupt_val = madt
        .get_entries::<tables::madt::InterruptSourceOverride>()
        .find(|entry| entry.source() == 1)
        .map(|entry| entry.global_system_interrupt())
        .unwrap_or(1);

    let processor_local_apic = madt
        .get_entries::<tables::madt::ProcessorLocalAPIC>()
        .next()
        .expect("no processor local APIC found")
        .apic_id();

    let mut entry = io_apic.ioredtbl(global_system_interrupt_val);
    entry.set_vector(InterruptIndex::Keyboard.as_u8());
    entry.set_delivery_mode(DeliveryMode::Fixed);
    entry.set_destination_mode(false);
    entry.set_pin_polarity(false);
    entry.set_trigger_mode(false);
    entry.set_masked(false);
    entry.set_destination(processor_local_apic);
    io_apic.set_ioredtbl(global_system_interrupt_val, entry);

    CHANNEL_HANDLE.call_once(|| add_system_port("sys.keyboard", add_listener, SYS_PORT_NO_RECEIVE));
}

pub extern "x86-interrupt" fn interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            let key: u64 = match key {
                DecodedKey::Unicode(character) => character as u64,
                DecodedKey::RawKey(key) => ' ' as u64,
            };

            use crate::process::messaging::{send, GenericMessage, MessageType};
            let sender = *CHANNEL_HANDLE
                .get()
                .expect("keyboard channel not initialized");
            for listener in LISTENERS.lock().iter() {
                send(
                    GenericMessage {
                        sender,
                        data: MessageType::Scalar(key, 0, 0, 0),
                    },
                    *listener,
                );
            }
            // match key {
            //     DecodedKey::Unicode('\n') => crate::gfx::framebuffer().confirm_input(),
            //     DecodedKey::Unicode('\u{8}') => crate::gfx::framebuffer().delete_input_char(),
            //     DecodedKey::Unicode(character) => {
            //         crate::gfx::framebuffer().add_input_char(character)
            //     }
            //     _ => {}
            // }
        }
    }

    LOCAL_APIC.get().unwrap().eoi();
}
