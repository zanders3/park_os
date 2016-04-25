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

	println!("Ready2");
	loop {}
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct Regs {
    pub sp: usize,
    pub ax: usize,
    pub bx: usize,
    pub cx: usize,
    pub dx: usize,
    pub di: usize,
    pub si: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub bp: usize,
    pub interrupt: usize,
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    //pub ss: usize,
}

#[no_mangle]
pub extern fn fault_handler(regs: &Regs) {
	macro_rules! printregs {
		($name:expr) => ({
			println!("  INT {:X}: {}", regs.interrupt, $name);
            println!("    CS:  {:08X}    IP:  {:08X}    FLG: {:08X}", regs.cs, regs.ip, regs.flags);
            println!("    SP:  {:08X}    BP:  {:08X}", regs.sp, regs.bp);
            println!("    AX:  {:08X}    BX:  {:08X}    CX:  {:08X}    DX:  {:08X}", regs.ax, regs.bx, regs.cx, regs.dx);
            println!("Extra stuff {:08X}", regs.sp);
            println!("    DI:  {:08X}    SI:  {:08X}", regs.di, regs.di);
		})
	}
	macro_rules! crashregs {
	    ($name:expr) => ({
	    	printregs!($name);
	    	loop { unsafe { asm!("hlt"); } }
	    })
	}

	match regs.interrupt {
		0x0 => crashregs!("Divide by zero exception"),
        0x1 => crashregs!("Debug exception"),
        0x2 => crashregs!("Non-maskable interrupt"),
        0x3 => crashregs!("Breakpoint exception"),
        0x4 => crashregs!("Overflow exception"),
        0x5 => crashregs!("Bound range exceeded exception"),
        0x6 => crashregs!("Invalid opcode exception"),
        0x7 => crashregs!("Device not available exception"),
        0x8 => crashregs!("Double fault"),
        0x9 => crashregs!("Coprocessor Segment Overrun"), // legacy
        0xA => crashregs!("Invalid TSS exception"),
        0xB => crashregs!("Segment not present exception"),
        0xC => crashregs!("Stack-segment fault"),
        0xD => crashregs!("General protection fault"),
        0xE => crashregs!("Page fault"),
        0x10 => crashregs!("x87 floating-point exception"),
        0x11 => crashregs!("Alignment check exception"),
        0x12 => crashregs!("Machine check exception"),
        0x13 => crashregs!("SIMD floating-point exception"),
        0x14 => crashregs!("Virtualization exception"),
        0x1E => crashregs!("Security exception"),
        _ => printregs!("Unknown interrupt")
	}
	println!("Returning now");
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
