mod area_frame_allocator;
mod table;
mod page;
mod entry;
mod pagetable;

pub use self::area_frame_allocator::AreaFrameAllocator;
use self::pagetable::remap_kernel;
use self::page::PhysicalAddress;
use multiboot2::BootInformation;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
	number: usize,
}

impl Frame {
	fn containing_address(address: usize) -> Frame {
		Frame{ number: address / PAGE_SIZE }
	}
	fn start_address(&self) -> PhysicalAddress {
		self.number * PAGE_SIZE
	}
	fn clone(&self) -> Frame {
		Frame { number: self.number }
	}
	fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
		FrameIter {
			start: start,
			end: end
		}
	}
}

struct FrameIter {
	start: Frame,
	end: Frame,
}

impl Iterator for FrameIter {
	type Item = Frame;

	fn next(&mut self) -> Option<Frame> {
		if self.start <= self.end {
			let frame = self.start.clone();
			self.start.number += 1;
			Some(frame)
		} else {
			None
		}
	}
}

pub trait FrameAllocator {
	fn allocate_frame(&mut self) -> Option<Frame>;
	fn deallocate_frame(&mut self, frame: Frame);
}

pub fn init_memory(boot_info: &BootInformation, multiboot_information_address: usize) {
	let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
	let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");

	let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
	let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();
	let multiboot_start = multiboot_information_address;
	let multiboot_end = multiboot_start + (boot_info.total_size as usize);
	println!("kernel_start: 0x{:x}, kernel_end: 0x{:x}\nmultiboot_start: 0x{:x}, multiboot_end: 0x{:x}",
		kernel_start, kernel_end, multiboot_start, multiboot_end);

	let mut frame_allocator = AreaFrameAllocator::new(
		kernel_start as usize, kernel_end as usize, multiboot_start, multiboot_end, memory_map_tag.memory_areas()
	);
	remap_kernel(&mut frame_allocator, &boot_info);
}

