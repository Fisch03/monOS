use x86_64::instructions::port::Port;

const INIT: u8 = 0x11;
const MODE_8086: u8 = 0x01;

struct Pic {
    command: Port<u8>,
    data: Port<u8>,
}

struct Pics {
    pic1: Pic,
    pic2: Pic,
}

impl Pics {
    fn new(offset: u8) -> Self {
        Pics {
            pic1: Pic {
                command: Port::new(0x20),
                data: Port::new(0x21),
            },
            pic2: Pic {
                command: Port::new(0xA0),
                data: Port::new(0xA1),
            },
        }
    }

    fn write_offset(&mut self, offset: u8) {
        unsafe {
            self.pic1.data.write(offset);
            self.pic2.data.write(offset + 8);
        }
    }

    fn configure_chaining(&mut self) {
        unsafe {
            self.pic1.data.write(4);
            self.pic2.data.write(2);
        }
    }

    fn write_both(&mut self, command: u8) {
        unsafe {
            self.pic1.command.write(command);
            self.pic2.command.write(command);
        }
    }

    fn write_mask(&mut self, mask: u8) {
        unsafe {
            self.pic1.data.write(mask);
            self.pic2.data.write(mask);
        }
    }
}

pub fn disable_pic() {
    let mut pics = Pics::new(0x20);

    let mut wait_port: Port<u8> = Port::new(0x80);
    let mut wait = || unsafe { wait_port.write(0) };

    pics.write_both(INIT);
    wait();

    pics.write_offset(0x20);
    wait();

    pics.configure_chaining();
    wait();

    pics.write_both(MODE_8086);
    wait();

    // mask all interrupts
    pics.write_mask(0b1111_1111);
}
