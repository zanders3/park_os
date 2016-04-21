use core::marker::PhantomData;

trait InOut {
	unsafe fn port_in(port: u16) -> Self;
	unsafe fn port_out(port: u16, val:Self);
}

impl InOut for u8 {
	unsafe fn port_in(port: u16) -> u8 {
		let val : u8 = 0u8;
		asm!("inb %dx, %al" :: "{=al}"(val), "{dx}"(port) :: "volatile");
		val
	}
	unsafe fn port_out(port: u16, val:u8) {
		asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(val) :: "volatile");
	}
}

impl InOut for u16 {
	unsafe fn port_in(port: u16) -> u16 {
		let val : u16 = 0u16;
		asm!("inw %dx, %ax" :: "{=ax}"(val), "{dx}"(port) :: "volatile");
		val
	}
	unsafe fn port_out(port: u16, val:u16) {
		asm!("outw %ax, %dx" :: "{dx}"(port), "{ax}"(val) :: "volatile");
	}
}

impl InOut for u32 {
	unsafe fn port_in(port: u16) -> u32 {
		let val : u32 = 0;
		asm!("inl %dx, %eax" :: "{=eax}"(val), "{dx}"(port) :: "volatile");
		val
	}
	unsafe fn port_out(port: u16, val: u32) {
		asm!("outl %eax, %dx" :: "{dx}"(port), "{eax}"(val) :: "volatile");
	}
}

struct Port<T : InOut> {
	port: u16,
	phantom: PhantomData<T>
}

impl<T : InOut> Port<T> {
	pub const unsafe fn new(port: u16) -> Port<T> {
		Port {
			port: port,
			phantom: PhantomData
		}
	}
	pub unsafe fn read(&mut self) -> T {
		T::port_in(self.port)
	}
	pub unsafe fn write(&mut self, val:T) {
		T::port_out(self.port, val);
	}
}

pub fn test() {
	/*unsafe {
		let mut port : Port<u16> = Port::new(0x60);
		let val = port.read();
		port.write(val);
	}*/
}
