use io::port::{Io, Port};

pub struct Ata {
}

impl Ata {
	pub const unsafe fn new() -> Ata {
		Ata {
		}
	}

	pub unsafe fn init_ata(&mut self) {
		/*let base = 0x1F0;
		let dev_ctrl = 0x3F6;
		let REG_CYL_LO = 4;
		let REG_CTL_HI = 5;
		let REG_DEVSEL = 6;
		let slavebit = 0;

		Port::new(base + REG_DEVSEL).write(0xA0 | slavebit << 4);
		Port::new(dev_ctrl).read();
		Port::new(dev_ctrl).read();
		Port::new(dev_ctrl).read();
		Port::new(dev_ctrl).read();
		let cl = Port::new(dev_ctrl+REG_CYL_LO).read();
		let ch = Port::new(dev_ctrl+REG_CTL_HI).read();
		println!("{} and {}", cl, ch);*/
	}
}
