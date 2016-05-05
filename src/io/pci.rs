use io::port::{Io, Port};


fn pci_address(bus : u32, slot : u32, func : u32, offset : u32) -> u32 {
	1 << 31 | (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xfc)
}

fn pci_read_word(bus : u32, slot : u32, func : u32, offset : u32) -> u16 {
	let address = pci_address(bus, slot, func, offset);
	unsafe {
		Port::new(0xCF8).write(address);
		Port::new(0xCFC).read()
	}
}

#[derive(Debug)]
struct Vendor {
	vendor : u16,
	device : u16
}

fn get_vendor(bus : u32, slot : u32) -> Option<Vendor> {
	let vendor = pci_read_word(bus, slot, 0, 0);
	if vendor == 0xFFFF {
		None
	} else {
		Some(Vendor {
			vendor: vendor,
			device: pci_read_word(bus, slot, 0, 2)
		})
	}
}

pub fn init_pci() {
	for bus in 0..256 {
		for slot in 0..32 {
			match get_vendor(bus, slot) {
				Some(vendor) => { 
					println!("PCI {} {}: {:?}", bus, slot, vendor); 
				},
				None => {}
			}
		}
	}
}

