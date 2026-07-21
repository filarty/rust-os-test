ENTRY(_start)

SECTIONS
{
    . = 2M;

    .text : ALIGN(4096)
    {
        *(.text._start)
        *(.text .text.*)
    }

    .rodata : ALIGN(4096)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4096)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4096)
    {
        _bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        _bss_end = .;
    }
}