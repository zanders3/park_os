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

use io::port::Io;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
	x86::enable_nxe_bit();
	x86::enable_write_protect_bit();
	
	vga_buffer::clear_screen();
	println!("Starting ParkOS");

	let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
	memory::init_memory(boot_info, multiboot_information_address);
	io::init_io();
    println!("Ready");

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
    pub flags: usize
}

#[no_mangle]
pub extern fn fault_handler(regs: &Regs) {
	let printregs = |name| {
		println!("  INT {:X}: {}", regs.interrupt, name);
        println!("    CS:  {:08X}    IP:  {:08X}    FLG: {:08X}", regs.cs, regs.ip, regs.flags);
        println!("    SP:  {:08X}    BP:  {:08X}", regs.sp, regs.bp);
        println!("    AX:  {:08X}    BX:  {:08X}    CX:  {:08X}    DX:  {:08X}", regs.ax, regs.bx, regs.cx, regs.dx);
        println!("    DI:  {:08X}    SI:  {:08X}", regs.di, regs.di);
        println!("HALT");
        loop { unsafe { asm!("hlt"); } }
	};

	match regs.interrupt {
		0x0 => printregs("Divide by zero exception"),
        0x1 => printregs("Debug exception"),
        0x2 => printregs("Non-maskable interrupt"),
        0x3 => printregs("Breakpoint exception"),
        0x4 => printregs("Overflow exception"),
        0x5 => printregs("Bound range exceeded exception"),
        0x6 => printregs("Invalid opcode exception"),
        0x7 => printregs("Device not available exception"),
        0x8 => printregs("Double fault"),
        0x9 => printregs("Coprocessor Segment Overrun"), // legacy
        0xA => printregs("Invalid TSS exception"),
        0xB => printregs("Segment not present exception"),
        0xC => printregs("Stack-segment fault"),
        0xD => printregs("General protection fault"),
        0xE => printregs("Page fault"),
        0x10 => printregs("x87 floating-point exception"),
        0x11 => printregs("Alignment check exception"),
        0x12 => printregs("Machine check exception"),
        0x13 => printregs("SIMD floating-point exception"),
        0x14 => printregs("Virtualization exception"),
        0x1E => printregs("Security exception"),
        0x20 => io::handle_timer_interrupt(),
        0x21 => unsafe {//keyboard interrupt
            let key_event = io::KEYBOARD.handle_keyboard_interrupt();
            if key_event.pressed && key_event.character != '\0' {
                vga_buffer::WRITER.lock().write_byte(key_event.character as u8);
            }
        },
        _ => printregs("Unknown interrupt"),
	}

    //Acknowledge interrupts from master and slave PIC
    if regs.interrupt >= 0x20 && regs.interrupt < 0x30 {
        if regs.interrupt >= 0x28 {
            unsafe { io::PICS.slave.command.write(0x20); }
        }
        unsafe { io::PICS.master.command.write(0x20); }
    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
	println!("\n\nPANIC in {} at line {}:", file, line);
	println!("	{}", fmt);
    println!("HALT");
    loop { unsafe { asm!("hlt"); } }
}
