#![feature(lang_items)]
#![feature(const_fn)]
#![feature(asm)]
#![no_std]

extern crate rlibc;
extern crate spin;
extern crate multiboot2;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod vga_buffer;
mod memory;
mod x86;
mod io;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
	x86::enable_nxe_bit();
	x86::enable_write_protect_bit();
	
	vga_buffer::clear_screen();
	println!("Starting ParkOS");

	let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
	memory::init_memory(boot_info, multiboot_information_address);
	println!("Ready");

	io::init_io();

	println!("Input!");
	loop {}
}

#[no_mangle]
pub extern fn fault_handler() {
	println!("ISR CALLED!");
	loop {}
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
	println!("\n\nPANIC in {} at line {}:", file, line);
	println!("	{}", fmt);
	loop{
		unsafe {
			asm!("hlt");
		}
	}
}
