/* Set the target architecture */
OUTPUT_ARCH(riscv)

/* Define the entry point function */
ENTRY(_start)

/* OpenSBI (M-mode firmware) drops us here in S-mode */
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    _kernel_start = .;

    /* .text.init must come first: it holds _start */
    .text : {
        *(.text.init)
        *(.text .text.*)
    }

    . = ALIGN(4096);
    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4096);
    .data : {
        /* gp points into the middle of the small-data area so that
           +/- 2KiB signed offsets cover all of .sdata/.sbss */
        PROVIDE(_global_pointer = . + 0x800);
        *(.sdata .sdata.*)
        *(.data .data.*)
    }

    . = ALIGN(4096);
    .bss : {
        . = ALIGN(8);
        _bss_start = .;
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        _bss_end = .;
    }

    /* Boot stack, grows down from _stack_top */
    . = ALIGN(16);
    _stack_bottom = .;
    . = . + 0x10000;    /* 64 KiB */
    _stack_top = .;

    _kernel_end = .;
}
