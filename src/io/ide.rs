use io::port::{Io, Port};
use io::pci::PciConfig;

#[derive(Copy,Clone)]
struct IdeDisk {
	bus_command:Port<u8>,
	bus_status:Port<u8>,
	data:Port<u16>,
	error:Port<u8>,
	sector_count:Port<u8>,
	sector0:Port<u8>,
	sector1:Port<u8>,
	sector2:Port<u8>,
	devsel:Port<u8>,
	status:Port<u8>,
	command:Port<u8>,
	alt_status:Port<u8>,
	master:bool
}

pub struct Ide {
	disks: [IdeDisk;4],
	num_disks: u8
}

pub static mut IDE: Ide = unsafe { Ide::new() };

impl Ide {
	pub const unsafe fn new() -> Ide {
		Ide {
			disks: [IdeDisk {
				bus_command:Port::empty(),
				bus_status:Port::empty(),
				data:Port::empty(),
				error:Port::empty(),
				sector_count:Port::empty(),
				sector0:Port::empty(),
				sector1:Port::empty(),
				sector2:Port::empty(),
				devsel:Port::empty(),
				status:Port::empty(),
				command:Port::empty(),
				alt_status:Port::empty(),
				master:false
			}; 4],
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
		let bar1 : u16 = select_port(pci.read(0x14), 0x3F6);
		let bar2 : u16 = select_port(pci.read(0x18), 0x170);
		let bar3 : u16 = select_port(pci.read(0x1C), 0x374);
		let bar4 : u16 = (pci.read(0x20) & 0xFFF0) as u16;

		let mut num_disks = 0;
		println!("- IDE {:X} {:X} {:X} {:X} {:X}", bar0, bar1, bar2, bar3, bar4);
		{
			let busmaster = bar4;
			let data = bar0;
			let control = bar1;
			let irq = 0xE;
			println!("    Primary Master");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, irq, true) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
			println!("    Primary Slave");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, irq, false) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
		}
		{
			let busmaster = bar4 + 8;
			let data = bar2;
			let control = bar3;
			let irq = 0xF;
			println!("    Secondary Master");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, irq, true) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
			println!("    Secondary Slave");
			if let Some(disk) = IdeDisk::new(busmaster, data, control, irq, false) {
				self.disks[num_disks] = disk;
				num_disks += 1;
			}
		}
		self.num_disks = num_disks as u8;
		println!("{} disks", self.num_disks);
	}
}

const ATA_CMD_IDENTIFY_PACKET : u8 = 0xA1;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

const ATA_SR_BSY: u8 = 0x80;//Busy
const ATA_SR_DF: u8  = 0x20;//Drive Write Fault
const ATA_SR_DRQ: u8 = 0x08;//Data Request Ready
const ATA_SR_ERR: u8 = 0x01;//Error

impl IdeDisk {
	pub fn new(busmaster:u16, base:u16, ctrl:u16, irq:u8, master:bool) -> Option<IdeDisk> {
		unsafe {
			let mut disk = IdeDisk {
				bus_command:Port::new(busmaster),
				bus_status:Port::new(busmaster + 2),
				data:Port::new(base),
				error:Port::new(base + 1),
				sector_count:Port::new(base + 2),
				sector0:Port::new(base + 3),
				sector1:Port::new(base + 4),
				sector2:Port::new(base + 5),
				devsel:Port::new(base + 6),
				status:Port::new(base + 7),
				command:Port::new(base + 7),
				alt_status:Port::new(ctrl + 2),
				master:master
			};
			if disk.identify() {
				Some(disk)
			} else {
				None
			}
		}
	}

	fn ata_write(&mut self, cmd:u8, block:u64, len:u16) {
		//Wait for busy status flag to clear
		while (self.alt_status.read() & ATA_SR_BSY) == ATA_SR_BSY {}

		//Select master or slave drive
		self.devsel.write(if self.master {
			0b11100000
		} else {
			0b11110000
		});

		//Wait 400ns for command to work (each read takes 100ns)
		self.alt_status.read();
		self.alt_status.read();
		self.alt_status.read();
		self.alt_status.read();
		
		//Wait for busy status flag to clear
		while (self.alt_status.read() & ATA_SR_BSY) == ATA_SR_BSY {}

		self.sector_count.write(len as u8);
		self.sector0.write(block as u8);
		self.sector1.write((block >> 8) as u8);
		self.sector2.write((block >> 16) as u8);

		self.command.write(cmd);
	}

	fn ide_poll(&mut self, check_error:bool) -> u8 {
		

		if check_error {
			let state = self.alt_status.read();
			println!("State2: {:X}", state);
			if state & ATA_SR_ERR == ATA_SR_ERR {
				return 2;
			}
			if state & ATA_SR_DF != 0 {
				return 1;
			}
			if state & ATA_SR_DRQ == 0 {
				return 3;
			}
		}
		0
	}

	fn identify(&mut self) -> bool {
		println!("\tIdentify");
		if self.alt_status.read() == 0xFF {
			println!("\tFloating bus");
			return false;
		}

		//Send IDENTIFY command
		self.ata_write(ATA_CMD_IDENTIFY, 0, 0);

		//Check status
		{
			let status = self.alt_status.read();
			if status == 0 {//No device
				return false;
			}
		}

		//Wait for busy status flag to clear
		while (self.alt_status.read() & ATA_SR_BSY) == ATA_SR_BSY {}

		//Check for errors
		{
			let status = self.alt_status.read();
			println!("\tStatus: {:X}", status);
			if (status & ATA_SR_ERR) == ATA_SR_ERR {
				//Error flag might mean we have an ATAPI device (cdrom)
				let cl = self.sector1.read();
				let ch = self.sector2.read();
				if (cl == 0x14 && ch == 0xEB) || (cl == 0x69 && ch == 0x96) {
					println!("\tType: ATAPI");
				} else {
					return false;
				}
				self.ata_write(ATA_CMD_IDENTIFY_PACKET, 0, 0);
			} else if (status & ATA_SR_DRQ) != ATA_SR_DRQ {
				println!("\tData request not ready?");
				return false;
			}
		}

		//Read description
		for word in 0..256 {
			let d = self.data.read();
			let a = ((d >> 8) as u8) as char;
			//if a != ' ' && a != '\0' {
				print!("{}", a);
			//}
			let b = (a as u8) as char;
			//if b != ' ' && b != '\0' {
				print!("{}", b);
			//}
		}
		println!("");

		true
	}
}


