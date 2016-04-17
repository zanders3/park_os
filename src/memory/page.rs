use memory::PAGE_SIZE;
use memory::Frame;
use memory::table::{Table, Level4};
use memory::FrameAllocator;
use memory::entry::*;

pub const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct Page {
	number: usize
}

impl Page {
	pub fn containing_address(address: VirtualAddress) -> Page {
		assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
			"invalid address: 0x{:x}", address);
		Page { number: address / PAGE_SIZE }
	}

	fn start_address(&self) -> usize {
		self.number * PAGE_SIZE
	}

	fn p4_index(&self) -> usize {
		(self.number >> 27) & 0o777 // bits 27-36 from 64 bits
	}
	fn p3_index(&self) -> usize {
		(self.number >> 18) & 0o777 // bits 18-27 from 64 bits
	}
	fn p2_index(&self) -> usize {
		(self.number >> 9) & 0o777 // bits 9-18 from 64 bits
	}
	fn p1_index(&self) -> usize {
		(self.number >> 0) & 0o777 // bits 0-9 from 64 bits
	}
}

pub struct PageTable {
	p4: *mut Table<Level4>,
}

impl PageTable {
	pub unsafe fn new_active() -> PageTable {
		PageTable {
			p4: 0xffffffff_fffff000 as *mut _,
		}
	}

	fn p4(&self) -> &Table<Level4> {
		unsafe { &*self.p4 }
	}

	fn p4_mut(&mut self) -> &mut Table<Level4> {
		unsafe { &mut *self.p4 }
	}

	//Translates a virtual page into a physical frame
	fn translate_page(&self, page: Page) -> Option<Frame> {
		use memory::entry::HUGE_PAGE;

		let p3 = self.p4().next_table(page.p4_index());

		//Start at top p4 table, lookup the p3 table, then lookup the p2 table, 
		//then lookup the p1 table and grab the physical frame pointed to
		p3.and_then(|p3| p3.next_table(page.p3_index()))
			.and_then(|p2| p2.next_table(page.p2_index()))
			.and_then(|p1| p1[page.p1_index()].pointed_frame()) 
			.or_else(|| {
				//if the PRESENT flag was missing OR we're dealing with a HUGE_PAGE for ANY of those entries
				p3.and_then(|p3| {
					let p3_entry = &p3[page.p3_index()];
					//What size of HUGE_PAGE are we dealing with?
					// 1 GiB page?
					if let Some(start_frame) = p3_entry.pointed_frame() {
						if p3_entry.flags().contains(HUGE_PAGE) {
							// address must be 1GiB aligned
							assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
							return Some(Frame {
								number: start_frame.number + page.p2_index() * ENTRY_COUNT + page.p1_index(),
							});
						}
					}
					// 2 MiB page?
					if let Some(p2) = p3.next_table(page.p3_index()) {
						let p2_entry = &p2[page.p2_index()];
						if let Some(start_frame) = p2_entry.pointed_frame() {
							if p2_entry.flags().contains(HUGE_PAGE) {
								// address must be 2MiB aligned
								assert!(start_frame.number % ENTRY_COUNT == 0);
								return Some(Frame {
									number: start_frame.number + page.p1_index()
								});
							}
						}
					}
					None
				})
			})
	}

	//Translates a virtual address into a physical address
	pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
		let offset = virtual_address % PAGE_SIZE;
		self.translate_page(Page::containing_address(virtual_address))
			.map(|frame| frame.number * PAGE_SIZE + offset)
	}

	//Modify the page tables to map a Page to a Physical Frame - this is going to set up a page table recursively
	//and point the hierarchy to the physical frame address
	pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A) where A : FrameAllocator {
		let mut p4 = self.p4_mut();
		let mut p3 = p4.next_table_create(page.p4_index(), allocator);
		let mut p2 = p3.next_table_create(page.p3_index(), allocator);
		let mut p1 = p2.next_table_create(page.p2_index(), allocator);

		assert!(p1[page.p1_index()].is_unused());
		p1[page.p1_index()].set(frame, flags | PRESENT);
	}

	//Modify the page tables to map a Page to a new Physical Frame
	pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A) where A : FrameAllocator {
		let frame = allocator.allocate_frame().expect("no frames available");
		self.map_to(page, frame, flags, allocator);
	}

	//Modify the page tables unmap a Page to a physical frame - this simply zeros the P1 page table entry for now
	fn unmap<A>(&mut self, page: Page, allocator: &mut A) where A : FrameAllocator {
		assert!(self.translate(page.start_address()).is_some());

		let p1 = self.p4_mut()
			.next_table_mut(page.p4_index())
			.and_then(|p3| p3.next_table_mut(page.p3_index()))
			.and_then(|p2| p2.next_table_mut(page.p2_index()))
			.expect("mapping code doesn't support huge pages");

		let frame = p1[page.p1_index()].pointed_frame().unwrap();
		p1[page.p1_index()].set_unused();
		allocator.deallocate_frame(frame);
		//TODO: deallocate P2, P3 pages if empty?
	}
}

pub fn test_paging<A>(allocator : &mut A) where A : FrameAllocator {
	let mut page_table = unsafe { PageTable::new_active() };
	println!("Some = {:?}", page_table.translate(0));
	println!("Some = {:?}", page_table.translate(4096));
	println!("Some = {:?}", page_table.translate(512 * 4096));
	println!("Some = {:?}", page_table.translate(300 * 512 * 4096));
	println!("None = {:?}", page_table.translate(512 * 512 * 4096));
	println!("Some = {:?}", page_table.translate(512 * 512 * 4096 - 1));


	let addr = 42 * 512 * 512 * 4096; // 42nd P3 entry
	let page = Page::containing_address(addr);
	let frame = allocator.allocate_frame().expect("no more frames");
	println!("None = {:?}, map to {:?}", page_table.translate(addr), frame);

	page_table.map_to(page, frame, EntryFlags::empty(), allocator);
	println!("Some = {:?}", page_table.translate(addr));
	println!("Next free frame = {:?}", allocator.allocate_frame());
}
