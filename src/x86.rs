
//Reads the CR0 register
pub unsafe fn cr0() -> u64 {
	let ret: u64;
	asm!("mov %cr0, $0" : "=r" (ret));
	ret
}

//Writes the CR0 register
pub unsafe fn cr0_write(val : u64) {
	asm!("mov $0, %cr0" :: "r" (val) : "memory");
}

//Reads the CR3 register - causes a general protection fault if not in kernel mode
pub unsafe fn cr3() -> u64 {
	let ret: u64;
	asm!("mov %cr3, $0" : "=r" (ret));
	ret
}

//Writes the CR3 register - causes a general protection fault if not in kernel mode
pub unsafe fn cr3_write(val : u64) {
	asm!("mov $0, %cr3" :: "r" (val) : "memory");
}

//Invalidate a given address in the TLB using the invlpg CPU instruction
pub unsafe fn flush_tlb(addr: usize) {
	asm!("invlpg ($0)" :: "r" (addr) : "memory");
}

//Invalidates the TLB completely
pub unsafe fn flush_tlb_all() {
	cr3_write(cr3());
}

const IA32_EFER: u32 = 0xc0000080;

//Write the 64 bits MSR register
pub unsafe fn wrmsr(msr: u32, value: u64) {
	let low = value as u32;
	let high = (value >> 32) as u32;
	asm!("wrmsr" :: "{ecx}" (msr), "{eax}" (low), "{edx}" (high) : "memory" : "volatile");
}

//Read the 64 bits MSR register
pub unsafe fn rdmsr(msr: u32) -> u64 {
	let low : u32;
	let high : u32;
	asm!("rdmsr" : "={eax}" (low), "={edx}" (high) : "{ecx}" (msr) : "memory" : "volatile");
	((high as u64) << 32) | (low as u64)
}

//Enable the WRITABLE page table flag 
pub fn enable_write_protect_bit() {
	let wp_bit = 1 << 16;
	unsafe { cr0_write(cr0() | wp_bit) };
}

//enable the NO_EXECUTE page table flag
pub fn enable_nxe_bit() {
	let nxe_bit = 1 << 11;
	unsafe {
		let efer = rdmsr(IA32_EFER);
		wrmsr(IA32_EFER, efer | nxe_bit);
	}
}
