
use io::port::{Io, Port};

#[derive(Debug)]
pub struct KeyEvent {
	pub character: char,
	pub pressed: bool,
	pub scancode: u8
}

static SCANCODES: [[char; 2]; 58] = [
	['\0', '\0'],
	['\x1B', '\x1B'],
	['1', '!'],
	['2', '@'],
	['3', '£'],
	['4', '$'],
	['5', '%'],
	['6', '^'],
	['7', '&'],
	['8', '*'],
	['9', '('],
	['0', ')'],
	['-', '_'],
	['=', '+'],
	['\x10', '\x10'],
	['\x11', '\x11'],
	['q', 'Q'],
	['w', 'W'],
	['e', 'E'],
	['r', 'R'],
	['t', 'T'],
	['y', 'Y'],
	['u', 'U'],
	['i', 'I'],
	['o', 'O'],
	['p', 'P'],
	['[', '{'],
	[']', '}'],
	['\n', '\n'],
	['\0', '\0'],
	['a', 'A'],
	['s', 'S'],
	['d', 'D'],
	['f', 'F'],
	['g', 'G'],
	['h', 'H'],
	['j', 'J'],
	['k', 'K'],
	['l', 'L'],
	[';', ':'],
	['\'', '"'],
	['`', '~'],
	['\0', '\0'],
	['\\', '|'],
	['z', 'Z'],
	['x', 'X'],
	['c', 'C'],
	['v', 'V'],
	['b', 'B'],
	['n', 'N'],
	['m', 'M'],
	[',', '<'],
	['.', '>'],
	['/', '?'],
	['\0', '\0'],
	['*', '*'],
	['\0', '\0'],
	[' ', ' ']
];

pub struct Keyboard {
	shift: bool,
	capslock: bool
}

impl Keyboard {
	pub const fn new() -> Keyboard {
		Keyboard {
			shift: false,
			capslock: false
		}
	}

	pub fn init_keyboard(&self) {
		unsafe {
			let mut keyb_port : Port<u8> = Port::new(0x64);
			let mut data_port : Port<u8> = Port::new(0x60);
			while (keyb_port.read() & 0x1) == 1 {
				println!("empty keyboard!");
				data_port.read();
			}
		}
	}

	fn parse_scancode(&mut self, scancode:u8) -> KeyEvent {
		let scancode_idx = scancode & 0x7F;

		match scancode {
			0x2A | 0x36 => { self.shift = true; },
			0xAA | 0xB6 => { self.shift = false; },
			0x3A => { self.capslock = !self.capslock; }
			_ => {}
		}

		let character = if scancode_idx < 58 {
			let shift_idx = if self.capslock ^ self.shift { 1 } else { 0 };
			SCANCODES[scancode_idx as usize][shift_idx]
		} else {
			'\0'
		};

		KeyEvent {
			character: character,
			pressed: scancode < 0x7F,
			scancode: scancode
		}
	}

	pub fn handle_keyboard_interrupt(&mut self) -> KeyEvent {
		let scancode:u8 = unsafe { Port::new(0x60).read() };
		self.parse_scancode(scancode)	
	}
}
