ENTRY(_boot)

__phys_start = 0x82000000;
__virt_start = 0xffffffffc2000000;
/* gigapage size */
__ram_len = 1024M;

/* __virt_stack is mapped to __phys_stack */
__virt_stack = 0xffffffffbfe00000;
__phys_stack = 0x83000000;
__stack_len = 2M;

SECTIONS {
    . = __phys_start;

    /* Boot routine code (_boot) and data (page tables). */
    .boot : {
        /* _boot must be loaded at exactly 0x82000000. */
        *(.boot.start)
        *(.boot.data)
    }

    . = __virt_stack;

	/* Succeeds the kernel in physical memory */
	.stack ALIGN(16) (NOLOAD) : AT(__phys_stack) {
		. += __stack_len;
		__stack_top = .;
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
