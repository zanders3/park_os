
pub struct MemBuffer {
	buffer: [u8;512]
}

impl MemBuffer {
	pub fn new() -> MemBuffer {
		MemBuffer {
			buffer: [0;512]
		}
	}
	pub fn len(&self) -> usize {
		self.buffer.len()
	}
	pub fn get_u8(&self, i:usize) -> u8 {
		self.buffer[i]
	}
	pub fn get_u16(&self, i:usize) -> u16 {
		(self.buffer[i+1] as u16) << 8 | (self.buffer[i] as u16)
	}
	pub fn get_u32(&self, i:usize) -> u32 {
		(self.buffer[i+3] as u32) << 24 | (self.buffer[i+2] as u32) << 16 | 
		(self.buffer[i+1] as u32) << 8 | (self.buffer[i] as u32)
	}
	pub fn set_u16(&mut self, i:usize, val:u16) {
		self.buffer[i] = (val & 0xFF) as u8;
		self.buffer[i+1] = (val >> 8) as u8;
	}
	pub fn get_slice(&self, i:usize, len:usize) -> &[u8] {
		&self.buffer[i..(i+len)]
	}
}