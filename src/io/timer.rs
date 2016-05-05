
//use io::port::{Io, Port};

pub fn init_timer()
{
	/*let hz = 10;
	let divisor : u32 = 1193180 / hz;
	unsafe {
		let mut command_port : Port<u8> = Port::new(0x43);
		command_port.write(0x36);
		let mut data_port : Port<u8> = Port::new(0x40);
		data_port.write((divisor & 0xFF) as u8);
		data_port.write((divisor >> 8) as u8);
	}*/
}

static mut timer_ticks : u64 = 0;

pub fn handle_timer_interrupt()
{
	unsafe { 
		timer_ticks = timer_ticks + 1;
		if timer_ticks % 18 == 0 {
			//println!("Tick");
		}
	}
}
