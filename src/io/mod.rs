pub mod port;
mod pic;
pub mod keyboard;
pub mod timer;

pub use io::port::{Io, Port};
pub use io::pic::Pics;
pub use io::keyboard::{Keyboard, KeyEvent};
pub use io::timer::handle_timer_interrupt;

pub static mut PICS: Pics = unsafe { Pics::new() };
pub static mut KEYBOARD: Keyboard = Keyboard::new();

pub fn init_io() {
	unsafe {
		PICS.init();
	}
	self::timer::init_timer();
	//Enable interrupts
	unsafe {
		asm!("sti");
	}
	unsafe { KEYBOARD.init_keyboard() };
}
