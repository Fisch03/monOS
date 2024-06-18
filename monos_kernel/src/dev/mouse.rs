use crate::acpi::tables;
use crate::interrupts::{
    apic::{
        io_apic::{DeliveryMode, IOAPIC},
        LOCAL_APIC,
    },
    InterruptIndex, InterruptStackFrame,
};
use crate::mem::Mapping;
use crate::process::messaging::ChannelHandle;
use crate::utils::BitField;

use alloc::vec::Vec;
use spin::{Lazy, Mutex};
use x86_64::instructions::port::Port;

static MOUSE: Lazy<Mutex<Mouse>> = Lazy::new(|| Mutex::new(Mouse::new()));

static LISTENERS: Lazy<Mutex<Vec<ChannelHandle>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn add_listener(handle: ChannelHandle) {
    LISTENERS.lock().push(handle);
}

pub fn init(madt: &Mapping<tables::MADT>, io_apic: &mut Mapping<IOAPIC>) {
    let global_system_interrupt_val = madt
        .get_entries::<tables::madt::InterruptSourceOverride>()
        .find(|entry| entry.source() == 12)
        .map(|entry| entry.global_system_interrupt())
        .unwrap_or(12);

    let processor_local_apic = madt
        .get_entries::<tables::madt::ProcessorLocalAPIC>()
        .next()
        .expect("no processor local APIC found")
        .apic_id();

    let mut entry = io_apic.ioredtbl(global_system_interrupt_val);
    entry.set_vector(InterruptIndex::Mouse.as_u8());
    entry.set_delivery_mode(DeliveryMode::Fixed);
    entry.set_destination_mode(false);
    entry.set_pin_polarity(false);
    entry.set_trigger_mode(false);
    entry.set_masked(false);
    entry.set_destination(processor_local_apic);
    io_apic.set_ioredtbl(global_system_interrupt_val, entry);

    MOUSE.lock().init().expect("failed to initialize mouse");
}

pub extern "x86-interrupt" fn interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let packet = unsafe { port.read() };

    if let Some(state) = MOUSE.lock().handle_packet(packet) {
        use crate::process::messaging::{send, Message};
        let mut message = Message {
            sender: 0,
            handle: ChannelHandle::new(0),
            data: (state.x as u64, state.y as u64, state.flags.0 as u64, 0),
        };
        for listener in LISTENERS.lock().iter() {
            message.handle = *listener;
            send(message.clone());
        }
    }

    LOCAL_APIC.get().unwrap().eoi();
}

#[derive(Debug)]
struct Mouse {
    command: Port<u8>,
    data: Port<u8>,

    packet_type: u8,
    state: MouseState,
}

#[derive(Debug)]
enum MouseError {
    WaitTimeout,
    NoResponse,
}

impl Mouse {
    const GET_STATUS: u8 = 0x20;
    const SET_STATUS: u8 = 0x60;

    const COMMAND_SET_DEFAULTS: u8 = 0xF6;
    const COMMAND_ENABLE_PACKET_STREAM: u8 = 0xF4;

    pub fn new() -> Self {
        Self {
            command: Port::new(0x64),
            data: Port::new(0x60),

            packet_type: 0,
            state: MouseState {
                x: 0,
                y: 0,
                flags: MouseFlags(0),
            },
        }
    }

    pub fn init(&mut self) -> Result<(), MouseError> {
        self.write_command(Self::GET_STATUS)?;
        let mut status = self.read_data()?;
        status.set_bit(1, true); // enable IRQ12
        status.set_bit(5, false); // disable mouse clock
        self.write_command(Self::SET_STATUS)?;
        self.write_data(status)?;
        self.send_command(Self::COMMAND_SET_DEFAULTS)?;
        self.send_command(Self::COMMAND_ENABLE_PACKET_STREAM)?;
        Ok(())
    }

    fn send_command(&mut self, command: u8) -> Result<(), MouseError> {
        self.write_command(0xD4)?;
        self.write_data(command)?;
        if self.read_data()? != 0xFA {
            return Err(MouseError::NoResponse);
        }
        Ok(())
    }

    pub fn handle_packet(&mut self, packet: u8) -> Option<MouseState> {
        match self.packet_type {
            0 => {
                let flags = MouseFlags(packet);
                if !flags.is_valid() {
                    return None;
                }
                self.state.flags = flags;
            }
            1 => {
                if !self.state.flags.x_overflow() {
                    self.state.x = if self.state.flags.x_sign() {
                        self.sign_extend(packet)
                    } else {
                        packet as i16
                    };
                }
            }
            2 => {
                if !self.state.flags.y_overflow() {
                    self.state.y = if self.state.flags.y_sign() {
                        self.sign_extend(packet)
                    } else {
                        packet as i16
                    };
                }
            }
            _ => unreachable!(),
        }
        let r = if self.packet_type == 2 {
            Some(self.state.clone())
        } else {
            None
        };

        self.packet_type = (self.packet_type + 1) % 3;

        r
    }

    #[inline]
    fn sign_extend(&self, value: u8) -> i16 {
        ((value as u16) | 0xFF00) as i16
    }

    fn read_data(&mut self) -> Result<u8, MouseError> {
        self.wait_read()?;
        Ok(unsafe { self.data.read() })
    }

    fn write_data(&mut self, data: u8) -> Result<(), MouseError> {
        self.wait_write()?;
        unsafe { self.data.write(data) };
        Ok(())
    }

    fn write_command(&mut self, command: u8) -> Result<(), MouseError> {
        self.wait_write()?;
        unsafe { self.command.write(command) };
        Ok(())
    }

    fn wait_read(&mut self) -> Result<(), MouseError> {
        for _ in 0..100_000 {
            let value = unsafe { self.command.read() };
            if value & 0x1 == 1 {
                return Ok(());
            }
        }

        Err(MouseError::WaitTimeout)
    }

    fn wait_write(&mut self) -> Result<(), MouseError> {
        for _ in 0..100_000 {
            let value = unsafe { self.command.read() };
            if value & 0x2 == 0 {
                return Ok(());
            }
        }

        Err(MouseError::WaitTimeout)
    }
}

#[derive(Debug, Clone)]
struct MouseState {
    pub x: i16,
    pub y: i16,
    pub flags: MouseFlags,
}
#[derive(Clone)]
struct MouseFlags(u8);

impl MouseFlags {
    const LEFT_BUTTON: usize = 0;
    const RIGHT_BUTTON: usize = 1;
    const MIDDLE_BUTTON: usize = 2;
    const ALWAYS_1: usize = 3;
    const X_SIGN: usize = 4;
    const Y_SIGN: usize = 5;
    const X_OVERFLOW: usize = 6;
    const Y_OVERFLOW: usize = 7;

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0.get_bit(Self::ALWAYS_1) == true
    }

    #[inline]
    pub fn left_button(&self) -> bool {
        self.0.get_bit(Self::LEFT_BUTTON)
    }
    #[inline]
    pub fn right_button(&self) -> bool {
        self.0.get_bit(Self::RIGHT_BUTTON)
    }
    #[inline]
    pub fn middle_button(&self) -> bool {
        self.0.get_bit(Self::MIDDLE_BUTTON)
    }

    #[inline]
    pub fn x_sign(&self) -> bool {
        self.0.get_bit(Self::X_SIGN)
    }
    #[inline]
    pub fn y_sign(&self) -> bool {
        self.0.get_bit(Self::Y_SIGN)
    }

    #[inline]
    pub fn x_overflow(&self) -> bool {
        self.0.get_bit(Self::X_OVERFLOW)
    }
    #[inline]
    pub fn y_overflow(&self) -> bool {
        self.0.get_bit(Self::Y_OVERFLOW)
    }
}

impl core::fmt::Debug for MouseFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("MouseFlags")
            .field("left_button", &self.left_button())
            .field("right_button", &self.right_button())
            .field("middle_button", &self.middle_button())
            .field("x_sign", &self.x_sign())
            .field("y_sign", &self.y_sign())
            .field("x_overflow", &self.x_overflow())
            .field("y_overflow", &self.y_overflow())
            .finish()
    }
}
