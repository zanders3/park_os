use io::port::Io;
use io::pci::PciConfig;
use io::ide_disk::IdeDisk;

pub struct Ide {
	disks: [IdeDisk;4],
	num_disks: u8
}

pub static mut IDE: Ide = unsafe { Ide::new() };

impl Ide {
	pub const unsafe fn new() -> Ide {
		Ide {
			disks: [IdeDisk::empty(); 4],
			num_disks: 0
		}
	}

	pub fn init_ide(&mut self, pci:&mut PciConfig) {
		//Turn ON the bus master flag in the command register
		let status = pci.read(0x4);
		pci.write(0x4, status | 0x4);

		let select_port = |port:u32,default:u16| -> u16 {
			match port {
				0x0 | 0x1 => default,
				_ => (default & 0xFFF0) as u16
			}
		};

		//Read the base pointers or use defaults if unspecified (http://wiki.osdev.org/IDE#Detecting_a_PCI_IDE_Controller)
		let bar0 : u16 = select_port(pci.read(0x10), 0x1F0);
		let bar1 : u16 = select_port(pci.read(0x14), 0x3F4);
		let bar2 : u16 = select_port(pci.read(0x18), 0x170);
		let bar3 : u16 = select_port(pci.read(0x1C), 0x374);
		let bar4 : u16 = (pci.read(0x20) & 0xFFF0) as u16;

		let mut num_disks = 0;
		//println!("- IDE {:X} {:X} {:X} {:X} {:X}", bar0, bar1, bar2, bar3, bar4);
		{
			let busmaster = bar4;
			let data = bar0;
			let control = bar1;
			//let irq = 0xE;
			println!("    Primary Master");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, true) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
			println!("    Primary Slave");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, false) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
		}
		{
			let busmaster = bar4 + 8;
			let data = bar2;
			let control = bar3;
			//let irq = 0xF;
			println!("    Secondary Master");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, true) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
			println!("    Secondary Slave");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, false) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
		}
		self.num_disks = num_disks as u8;
		println!("{} disks", self.num_disks);
	}

	pub fn get_disk(&mut self) -> Option<&mut IdeDisk> {
		if self.num_disks > 0 {
			Some(&mut self.disks[0])
		} else {
			None
		}
	}
}


