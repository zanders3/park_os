pub mod port;
mod pic;
pub mod keyboard;
pub mod timer;

pub use io::port::{Io, Port};
pub use io::pic::Pics;
pub use io::keyboard::{Keyboard, KeyEvent};
pub use io::timer::handle_timer_interrupt;

pub static mut PICS: Pics = unsafe { Pics::new() };
pub static mut KEYBOARD: Keyboard = unsafe { Keyboard::new() };

pub fn init_io() {
	unsafe {
		PICS.init();
	}
	println!("Init timer");
	self::timer::init_timer();
	println!("Enabling interrupts...");
	unsafe {
		asm!("sti");
	}
	println!("Init keyboard");
	unsafe { KEYBOARD.init_keyboard() };
}
