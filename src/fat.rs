
use io::ide::IdeDisk;

pub struct FatFS {
}

pub static mut FS:FatFS = FatFS {};

impl FatFS {
	pub fn init_fs(&mut self, disk:&mut IdeDisk) {
		
	}
}
