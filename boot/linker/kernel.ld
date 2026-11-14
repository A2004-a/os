ENTRY(kernel_entry)

SECTIONS {
    . = 0xFFFF800000200000;

    .text ALIGN(4K) : AT(ADDR(.text) - 0xFFFF800000000000) {
        KEEP(*(.text.entry))
        *(.text .text.*)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - 0xFFFF800000000000) {
        *(.rodata .rodata.*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - 0xFFFF800000000000) {
        *(.data .data.*)
    }

    .bss ALIGN(4K) : AT(ADDR(.bss) - 0xFFFF800000000000) {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.comment)
    }
}
