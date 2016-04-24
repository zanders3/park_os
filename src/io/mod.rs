mod port;
mod pic;

use spin::Mutex;
use io::pic::ChainedPics;

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });

pub fn init_io() {
	println!("Here we go...");
	unsafe {
		PICS.lock().init();
	}
	println!("Enabling interrupts...");
	unsafe {
		asm!("sti");
	}
}
