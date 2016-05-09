use core::marker::PhantomData;


pub trait Io<T> {
    fn read(&self) -> T;
    fn write(&mut self, value: T);
}

#[derive(Copy,Clone)]
pub struct Port<T> {
	port: u16,
	phantom: PhantomData<T>
}

impl Io<u8> for Port<u8> {
    /// Read
    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("in $0, $1" : "={al}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    /// Write
    fn write(&mut self, value: u8) {
        unsafe {
            asm!("out $1, $0" : : "{al}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }
}

impl Io<u16> for Port<u16> {
    /// Read
    fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("in $0, $1" : "={ax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    /// Write
    fn write(&mut self, value: u16) {
        unsafe {
            asm!("out $1, $0" : : "{ax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }
}

impl Io<u32> for Port<u32> {
    /// Read
    fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("in $0, $1" : "={eax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    /// Write
    fn write(&mut self, value: u32) {
        unsafe {
            asm!("out $1, $0" : : "{eax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }
}

impl<T> Port<T> {
	pub const unsafe fn new(port: u16) -> Port<T> {
		Port {
			port: port,
			phantom: PhantomData
		}
	}
    pub const fn empty() -> Port<T> { Port{ port:0,phantom:PhantomData } }
}