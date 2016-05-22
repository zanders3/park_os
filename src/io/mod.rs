pub mod port;
mod pic;
mod pci;
pub mod ide;
pub mod ide_disk;
pub mod keyboard;
pub mod timer;
pub mod membuffer;

pub use io::port::{Io, Port};
pub use io::pic::Pics;
pub use io::keyboard::{Keyboard, KeyEvent};
pub use io::timer::handle_timer_interrupt;
pub use io::ide_disk::IdeDisk;
pub use io::membuffer::MemBuffer;

pub static mut PICS: Pics = unsafe { Pics::new() };
pub static mut KEYBOARD: Keyboard = Keyboard::new();

pub fn init_io() {
	unsafe {
		PICS.init();
		self::timer::init_timer();
		//Enable interrupts
		asm!("sti");
		KEYBOARD.init_keyboard();
		self::pci::init_pci();
	}
}
