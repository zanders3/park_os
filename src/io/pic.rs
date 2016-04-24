use io::port::Port;

//Programmable interrupt controller
// http://wiki.osdev.org/8259_PIC
pub struct Pic {
	offset: u8,
	command: Port<u8>,
	data: Port<u8>
}

pub struct ChainedPics {
	pics: [Pic; 2]
}

const ICW1_ICW4 : u8 = 0x01;/* ICW4 (not) needed */
const ICW1_INIT : u8 = 0x10;/* Initialization - required! */

const ICW4_8086 : u8 = 0x01;/* 8086/88 (MCS-80/85) mode */

impl ChainedPics {
	pub const unsafe fn new(offset1:u8, offset2:u8) -> ChainedPics {
		ChainedPics {
			pics : [
				Pic {
					offset: offset1,
					command: Port::new(0x20),
					data: Port::new(0x21)
				},
				Pic {
					offset: offset2,
					command: Port::new(0xA0),
					data: Port::new(0xA1)
				}
			]
		}
	}

	pub unsafe fn init(&mut self) {
		let mut wait_port: Port<u8> = Port::new(0x80);
		let mut wait = || { wait_port.write(0) };

		let saved_mask1 = self.pics[0].data.read();
		let saved_mask2 = self.pics[1].data.read();

		self.pics[0].command.write(ICW1_INIT + ICW1_ICW4);//starts the init sequence in cascade mode
		wait();
		self.pics[1].command.write(ICW1_INIT + ICW1_ICW4);
		wait();
		self.pics[0].data.write(self.pics[0].offset);//ICW2: Master PIC vector offset
		wait();
		self.pics[1].data.write(self.pics[1].offset);//ICW2: Slave PIC vector offset
		wait();
		self.pics[0].data.write(4);//ICW3: tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
		wait();
		self.pics[1].data.write(2);// ICW3: tell Slave PIC its cascade identity (0000 0010)
		wait();

		self.pics[0].data.write(ICW4_8086);
		wait();
		self.pics[1].data.write(ICW4_8086);
		wait();

		self.pics[0].data.write(saved_mask1);
		self.pics[1].data.write(saved_mask2);
	}
}
