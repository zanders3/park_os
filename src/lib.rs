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
use memory::FrameAllocator;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
	vga_buffer::clear_screen();
	println!("Starting ParkOS");

	let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
	let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

	println!("Memory areas:");
	for area in memory_map_tag.memory_areas() {
		println!("	start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
	}

	let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
	println!("Kernel sections:");
	for section in elf_sections_tag.sections() {
		println!("	addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", section.addr, section.size, section.flags);
	}

	let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
	let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();
	let multiboot_start = multiboot_information_address;
	let multiboot_end = multiboot_start + (boot_info.total_size as usize);
	println!("kernel_start: 0x{:x}, kernel_end: 0x{:x}\nmultiboot_start: 0x{:x}, multiboot_end: 0x{:x}",
		kernel_start, kernel_end, multiboot_start, multiboot_end);

	let mut frame_allocator = memory::AreaFrameAllocator::new(
		kernel_start as usize, kernel_end as usize, multiboot_start, multiboot_end, memory_map_tag.memory_areas()
	);
	memory::test_paging(&mut frame_allocator);
	memory::remap_kernel(&mut frame_allocator, boot_info);
	println!("Done");
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
