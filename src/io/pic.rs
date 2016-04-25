use io::port::Port;

//Programmable interrupt controller
// http://wiki.osdev.org/8259_PIC
pub struct Pic {
	offset: u8,
	pub command: Port<u8>,
	pub data: Port<u8>
}

pub struct Pics {
	pub master: Pic,
	pub slave: Pic
}

const ICW1_ICW4 : u8 = 0x01;/* ICW4 (not) needed */
const ICW1_INIT : u8 = 0x10;/* Initialization - required! */

const ICW4_8086 : u8 = 0x01;/* 8086/88 (MCS-80/85) mode */

impl Pics {
	pub const unsafe fn new() -> Pics {
		Pics {
			master: Pic {
				offset: 0x20,
				command: Port::new(0x20),
				data: Port::new(0x21)
			},
			slave: Pic {
				offset: 0x28,
				command: Port::new(0xA0),
				data: Port::new(0xA1)
			}
		}
	}

	pub unsafe fn init(&mut self) {
		let mut wait_port: Port<u8> = Port::new(0x80);
		let mut wait = || { wait_port.write(0) };

		let saved_mask1 = self.master.data.read();
		let saved_mask2 = self.slave.data.read();

		self.master.command.write(ICW1_INIT + ICW1_ICW4);//starts the init sequence in cascade mode
		wait();
		self.slave.command.write(ICW1_INIT + ICW1_ICW4);
		wait();
		self.master.data.write(self.master.offset);//ICW2: Master PIC vector offset
		wait();
		self.slave.data.write(self.slave.offset);//ICW2: Slave PIC vector offset
		wait();
		self.master.data.write(4);//ICW3: tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
		wait();
		self.slave.data.write(2);// ICW3: tell Slave PIC its cascade identity (0000 0010)
		wait();

		self.master.data.write(ICW4_8086);
		wait();
		self.slave.data.write(ICW4_8086);
		wait();

		self.master.data.write(saved_mask1);
		self.slave.data.write(saved_mask2);
	}
}
