
use io::port::{Io, Port};

/*struct PS2Controller {
	data: Port<u8>,
	status: Port<u8>,
	command: Port<u8>
}

impl PS2Controller {
	pub fn new() -> PS2Controller {
		data: Port::new(0x60),
		command: Port::new(0x64)
	}

	pub fn write_command(val:u8) {
		self.command.write(val);
	}

	pub fn read_status() -> u8 {
		self.command.read()
	}

	pub fn write_data(val:u8) {
		self.data.write(val);
	}

	pub fn read_data() -> u8 {
		self.data.read()
	}
}

static mut PS2: PS2Controller = unsafe { PS2Controller::new() };*/

pub fn handle_keyboard_interrupt() {
	let mut scancode:u8 = unsafe { Port::new(0x60).read() };
	println!("pressed {}", scancode);	
}

pub fn init_keyboard() {
	//Do nothing?
}
