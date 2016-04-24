global idtr
global setup_idt

extern gdt64.code

section .text
bits 64

; Define IDT code for 255 interrupt handlers - putting interrupt code into .int_code
interrupts:
.first:
	mov [.int_code], byte 0
	jmp qword .handle
.second:
%assign i 1
%rep 255
	mov [.int_code], byte i
	jmp qword .handle
%assign i i+1
%endrep

.handle:
	push rbp ; Save all registers
	push r15
	push r14
	push r13
	push r12
	push r11
	push r10
	push r9
	push r8
	push rsi
	push rdi
	push rdx
	push rcx
	push rbx
	push rax

	mov rsi, rsp ; Save stack pointer
	push rsi

	mov rdi, qword [.int_code] ; Push int_code

	extern fault_handler
	call fault_handler ; Call rust fault handler

	pop rsp ; Pop stack pointer

	pop rax ; Restore all registers
	pop rbx
	pop rcx
	pop rdx
	pop rdi
	pop rsi
	pop r8
	pop r9
	pop r10
	pop r11
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp

	iretq

; Function pointer to rust interrupt handler (set in io::init_io)
.handler: dq 0

; IDTR definition
idtr:
	dw (idt.end - idt) + 1 ; idt limit - maximum addressable size in table
	dq idt ; pointer to idt

%define BASE_OF_SECTION 0x101160 ;terrifying HACKX - if interrupts go horribly wrong its going to be this base address!
%define SIZE_OF_INTCODE (interrupts.second-interrupts.first)

; IDT definition starts here
idt:
%assign i 0

; create 255 IDT entries
%rep 255
	; interrupt handler code is located at interrupts + sizeof(interrupt code) * i
	; we are in assembly so this should be located in the first segment so the middle and upper fields can be 0
	dw ((BASE_OF_SECTION + (SIZE_OF_INTCODE*i)) & 0xFFFF) ;offsetl
	dw gdt64.code ; pointer to GDT code segment - selector
	db 0 ; zero
	db (1<<7) | 0xE ; PRESENT | INTERRUPT64 - type and attributes
	dw ((BASE_OF_SECTION + (SIZE_OF_INTCODE*i)) >> 16) ; offset middle bits
	dd 0 ; offset higher bits
	dd 0 ; always 0
%assign i i+1
%endrep
.end:
;end of IDT
