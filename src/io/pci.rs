use io::port::{Io, Port};
use io::ide::IDE;

pub struct PciConfig {
	bus: u8,
	slot: u8,
	func: u8,
	addr: Port<u32>,
	data: Port<u32>
}

impl PciConfig {
	pub fn new(bus:u8, slot:u8, func:u8) -> PciConfig {
		PciConfig {
			bus: bus,
			slot: slot,
			func: func,
			addr: unsafe { Port::new(0xCF8) },
			data: unsafe { Port::new(0xCFC) }
		}
	}

	fn set_address(&mut self, offset:u8) {
		let address = 1 << 31 | ((self.bus as u32) << 16) | ((self.slot as u32) << 11) | 
			((self.func as u32) << 8) | ((offset as u32) & 0xfc);
		self.addr.write(address);
	}

	pub fn read(&mut self, offset:u8) -> u32 {
		self.set_address(offset);
		self.data.read()
	}

	pub fn write(&mut self, offset:u8, val:u32) {
		self.set_address(offset);
		self.data.write(val);
	}
}

#[derive(Debug)]
struct PciDevice {
	class_id: u8,
	subclass_id: u8,
	interface_id: u8,
	vendor_code: u16,
	device_code: u16,
	header_type: u8
}

pub fn init_pci() {
	for bus in 0..256 {
		for slot in 0..32 {
			for func in 0..8 {
				let mut pci = PciConfig::new(bus as u8, slot as u8, func as u8);
				let id = pci.read(0x0);
				if (id & 0xFFFF) != 0xFFFF {
					let class = pci.read(0x8);
					let bist = pci.read(0xC);
					let pci_device = PciDevice {
						class_id: ((class >> 24) & 0xFF) as u8,
						subclass_id: ((class >> 16) & 0xFF) as u8,
						interface_id: ((class >> 8) & 0xFF) as u8,
						vendor_code: (id & 0xFFFF) as u16,
						device_code: ((id >> 16) & 0xFFFF) as u16,
						header_type: ((bist >> 16) & 0xFF) as u8
					};

					let val = match pci_device.class_id {
						0x1 => match pci_device.subclass_id {
							0x1 => "IDE",
							0x2 => "Floppy Disk",
							0x5 => "ATA",
							0x6 => "SATA",
							_ => "Unknown Mass Storage"
						},
						0x2 => match pci_device.subclass_id {
							0x0 => "Ethernet",
							_ => "Unknown Network Device"
						},
						0x3 => match pci_device.subclass_id {
							0x0 => "VGA Controller",
							_ => "Display Controller"
						},
						0x4 => "Multimedia Controller",
						0x5 => "Memory Controller",
						0x6 => "Bridge Device",
						_ => "Unknown Device"
					};
					println!("PCI {} {} {}: {}", bus, slot, func, val);

					match (pci_device.class_id, pci_device.subclass_id) {
						(0x1,0x1) => { unsafe { IDE.init_ide(&mut pci); } },
						(_,_) => {}
					}
				}
			}
		}
	}
}

