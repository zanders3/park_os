mod port;
mod pic;

pub use io::pic::Pics;

pub static mut PICS: Pics = unsafe { Pics::new() };

pub fn init_io() {
	println!("Here we go...");
	unsafe {
		PICS.init();
	}
	println!("Enabling interrupts...");
	unsafe {
		asm!("sti");
	}
}
