global start
extern long_mode_start

section .text
bits 32
start:
	; setup small stack
	mov esp, stack_top

	call check_multiboot
	call check_cpuid
	call check_long_mode

	call set_up_page_tables
	call enable_paging
	call set_up_sse

	; load the 64-bit GDT
	lgdt [gdt64.pointer]

	; update selectors
	mov ax, gdt64.data
	mov ss, ax ; stack selector
	mov ds, ax ; data selector
	mov es, ax ; extra selector

	jmp gdt64.code:long_mode_start

    mov al, "3"
    jmp error
    hlt

; Prints 'ERR: ' and an error code to the screen then hangs
; parameter: error code (ascii) in al
error:
	mov dword [0xb8000], 0x4f524f45
	mov dword [0xb8004], 0x4f3a4f52
	mov dword [0xb8008], 0x4f204f20
	mov byte  [0xb800a], al
	hlt

; Check that we really were loaded by a multiboot complaint bootloader
check_multiboot:
	cmp eax, 0x36d76289
	jne .no_multiboot
	ret
.no_multiboot:
	mov al, "0"
	jmp error

; Check that CPUID is supported by attempting to flip the ID bit (bit 21) in
; the FLAGS register. If we can flip it, CPUID is available.
check_cpuid:
	; Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
    ; back if it was ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit wasn't
    ; flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp error

; Check if extended processor info is available (long mode)
check_long_mode:
	mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov al, "2"
    jmp error

; Setup the page tables defined in bss section
set_up_page_tables:
	; map first P4 entry to P3 table
	mov eax, p3_table
	or eax, 0b11 ; add present + writeable flags
	mov [p4_table], eax

	; map first P3 entry to P2 table
	mov eax, p2_table
	or eax, 0b11 ; add present + writeable flags
	mov [p3_table], eax

	; map each P2 entry to 2MB page
	mov ecx, 0 ; counter

.map_p2_table:
	; map each P2 entry to 2MiB * ecx memory location
	mov eax, 0x200000 ; 2MiB
	mul ecx,
	or eax, 0b10000011 ; add present + writeable + huge flags
	mov [p2_table + ecx * 8], eax ; map entry

	inc ecx
	cmp ecx, 512
	jne .map_p2_table ; if ecx < 512 map next entry

	ret

; Activate paging on the CPU
enable_paging:
	; load P4 to cr3 register (where the CPU looks for page table info)
	mov eax, p4_table
	mov cr3, eax

	; enable physical address extension mode
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; set the long mode bit in the EFER model specific register
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	; enable paging in the cr0 register
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

	ret

; Check for SSE and enable it. If unsupported throw error "a".
set_up_sse:
	; check for SSE
	mov eax, 0x1
	cpuid
	test edx, 1<<25
	jz .no_sse

	;enable SSE
	mov eax, cr0
	and ax, 0xFFFB ; clear coprocessor emulation CR0.EM
	or ax, 0x2     ; set coprocessor monitoring CR0.MP
	mov cr0, eax
	mov eax, cr4
	or ax, 3 << 9  ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
	mov cr4, eax

	ret
.no_sse
	mov al, "a"
	jmp error

section .rodata
; 64-bit Global descriptor table
gdt64:
	dq 0 ; zero entry
.code: equ $ - gdt64
	dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data: equ $ - gdt64
	dq (1<<44) | (1<<47) | (1<<41) ; data segment
.pointer:
	dw $ - gdt64 - 1
	dq gdt64

section .bss
; Page tables
align 4096
p4_table:
	resb 4096
p3_table:
	resb 4096
p2_table:
	resb 4096
; Small initial stack to get us to rust
stack_bottom:
	resb 64
stack_top:
