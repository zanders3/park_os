use io::port::{Io, Port};
use io::pci::PciConfig;
use io::membuffer::MemBuffer;

#[derive(Copy,Clone,Debug)]
enum DiskType {
	Unknown,
	ATA,
	ATAPI
}

#[derive(Copy,Clone,Debug)]
enum AccessType {
	Unknown,
	LBA28,
	LBA48
}

#[derive(Copy,Clone)]
pub struct IdeDisk {
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
	disk_type:DiskType,
	access_type:AccessType,
	num_sectors:u64,
	master:bool
}

const ATA_CMD_READ_PIO: u8 = 0x20;
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_IDENTIFY_PACKET : u8 = 0xA1;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

const ATA_SR_BSY: u8 = 0x80;//Busy
const ATA_SR_DF: u8  = 0x20;//Drive Write Fault
const ATA_SR_DRQ: u8 = 0x08;//Data Request Ready
const ATA_SR_ERR: u8 = 0x01;//Error

impl IdeDisk {
	pub const fn empty() -> IdeDisk {
		IdeDisk {
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
			disk_type:DiskType::Unknown,
			access_type:AccessType::Unknown,
			num_sectors:0,
			master:false
		}
	}

	pub fn new(busmaster:u16, base:u16, ctrl:u16, master:bool) -> Option<IdeDisk> {
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
				disk_type:DiskType::Unknown,
				access_type:AccessType::Unknown,
				num_sectors:0,
				master:master
			};
			if disk.identify() {
				Some(disk)
			} else {
				println!("\t\tNot Connected");
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

	fn print_range(min:usize,max:usize,data:&[u16]) {
		for i in min..max {
			let d = data[i];
            let a = ((d >> 8) as u8) as char;
            if a != ' ' && a != '\0' {
                print!("{}", a);
            }
            let b = (d as u8) as char;
            if b != ' ' && b != '\0' {
                print!("{}", b);
            }
		}
	}

	fn identify(&mut self) -> bool {
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
			//println!("\t\tStatus: {:X}", status);
			if (status & ATA_SR_ERR) == ATA_SR_ERR {
				//Error flag might mean we have an ATAPI device (cdrom)
				let cl = self.sector1.read();
				let ch = self.sector2.read();
				if (cl == 0x14 && ch == 0xEB) || (cl == 0x69 && ch == 0x96) {
					self.disk_type = DiskType::ATAPI;
				} else {
					//Not an ATAPI device!
					return false;
				}
				//Ask the ATAPI to identify itself
				self.ata_write(ATA_CMD_IDENTIFY_PACKET, 0, 0);
			} else if (status & ATA_SR_DRQ) != ATA_SR_DRQ {
				println!("\tData request not ready?");
				return false;
			} else {
				self.disk_type = DiskType::ATA;
			}
		}

		//Read in the identity data
		let mut data : [u16;256] = [0;256];
		for i in 0..256 {
			data[i] = self.data.read();
		}

		//Print out disk info
		print!("\t\tType: {:?} Serial: ", self.disk_type);
		IdeDisk::print_range(10, 20, &data);
		print!(" Firmware: ");
		IdeDisk::print_range(23, 27, &data);
		print!(" Model: ");
		IdeDisk::print_range(27, 47, &data);
		println!("");

		//the total number of 48 bit addressable sectors on the drive
		self.num_sectors = 
			(data[100] as u64) | 
			((data[101] as u64) << 16) |
			((data[102] as u64) << 32) |
			((data[103] as u64) << 48);
		//if >0 then LBA48 is 'probably' supported? (http://wiki.osdev.org/ATA_PIO_Mode)
		if self.num_sectors > 0 {
			self.access_type = AccessType::LBA48;
		} else {
			self.num_sectors =
				(data[60] as u64) |
				((data[61] as u64) << 16);
			if self.num_sectors > 0 {
				self.access_type = AccessType::LBA28;
			}
		}

		println!("\t\tSize: {} MB", (self.num_sectors / 2048) as usize);

		true
	}

	fn ata_pio(&mut self, write:bool, block:u64, buffer:&mut MemBuffer) -> Result<usize, &'static str> {
		let sector_count = 1;

		self.ata_write(if write {
			ATA_CMD_WRITE_PIO
		} else {
			ATA_CMD_READ_PIO
		}, block, sector_count);

		let mut num_read : usize = 0;
		for _ in 0..sector_count as usize {
			//Wait for busy status flag to clear
			while (self.alt_status.read() & ATA_SR_BSY) == ATA_SR_BSY {}

			//Check for errors
			let state = self.alt_status.read();
			if (state & ATA_SR_ERR) == ATA_SR_ERR {
				return Err("Read/write Error");
			} else if (state & ATA_SR_DF) == ATA_SR_DF {
				return Err("Drive Fault");
			} else if (state & ATA_SR_DRQ) != ATA_SR_DRQ {
				return Err("Expected Data Request Ready");
			}

			if write {
				return Err("Not implemented ;)");
			} else {
				for _ in 0..256 {
					buffer.set_u16(num_read, self.data.read());
					num_read += 2;
				}
			}
		}

		Ok(num_read)
	}

	pub fn read(&mut self, block:u64, buffer:&mut MemBuffer) -> Result<usize, &'static str> {
		self.ata_pio(false, block, buffer)
	}
}
