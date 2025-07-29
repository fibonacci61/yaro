ENTRY(_boot)

__phys_ram_start = 0x80000000;
__virt_ram_start = 0xffffffffc0000000;
__kernel_offset = 0x2000000;

__phys_start = __phys_ram_start + __kernel_offset;
__virt_start = __virt_ram_start + __kernel_offset;
/* gigapage size */
__ram_len = 1024M;

/* __virt_stack is mapped to __phys_stack */
__pheap_len = 2M;
__stack_len = 2M;

__virt_stack = __virt_ram_start - __stack_len;
__virt_pheap = __virt_stack - __pheap_len;

__phys_pheap = 0x83000000;
__phys_stack = __phys_pheap + __pheap_len;

SECTIONS {
    . = __phys_start;

    /* Boot routine code (_boot) and data (page tables). */
    .boot : {
        /* _boot must be loaded at exactly 0x82000000. */
        *(.boot.start)
        *(.boot.data)
    }

    . = __virt_pheap;

	/* Succeeds the kernel in physical memory */
    .pheap ALIGN(16) (NOLOAD) : AT(__phys_pheap) {
        . += __pheap_len;
    }

    . = __virt_stack;

	.stack ALIGN(16) (NOLOAD) : AT(__phys_stack) {
		. += __stack_len;
	}

    /* .boot is still part of our virtual memory */
    . = __virt_start + SIZEOF(.boot);

    .text : AT(__phys_start + SIZEOF(.boot)) {
        *(.text .text.*)
    }

    .rodata ALIGN(8) : {
        *(.rodata .rodata.*)
    }

	.data ALIGN(8) : {
		*(.data .data.*)
	}

	/* I'm not sure why these two are here */

	.eh_frame_hdr : {
        KEEP(*(.eh_frame_hdr))
	}

	.eh_frame : {
        KEEP(*(.eh_frame))
	}

	/* Must be zeroed out at runtime */
	.bss ALIGN(8) (NOLOAD) : {
		_sbss = .;
		*(.bss .bss.*)
		_ebss = .;
	}
}
