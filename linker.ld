/* src/arch/x86_64/linker.ld */

ENTRY(_start)

SECTIONS {
  . = 0x800000000;

  .rodata :
  {
    *(.rodata .rodata.*)
    . = ALIGN(0x1000);
  }

  .eh_frame_hdr :
  {
    *(.eh_frame_hdr)
    . = ALIGN(0x1000);
  }

  .eh_frame :
  {
    *(.eh_frame)
    . = ALIGN(0x1000);
  }

  .text :
  {
    *(.text .text.*)
    . = ALIGN(0x1000);
  }

  .data.rel.ro :
  {
    *(.data.rel.ro)
    . = ALIGN(0x1000);
  }

  .got :
  {
    *(.got)
    . = ALIGN(0x1000);
  }

  .data :
  {
    *(.data .data.*)
    . = ALIGN(0x1000);
  }

  .bss :
  {
    *(.bss .bss.*)
    . = ALIGN(0x1000);
  }

  .debug_loc :
  {
    *(.debug_loc)
    . = ALIGN(0x1000);
  }

  .debug_abbrev :
  {
    *(.debug_abbrev)
    . = ALIGN(0x1000);
  }

  .debug_info :
  {
    *(.debug_info)
    . = ALIGN(0x1000);
  }

  .debug_aranges :
  {
    *(.debug_aranges)
    . = ALIGN(0x1000);
  }

  .debug_ranges :
  {
    *(.debug_ranges)
    . = ALIGN(0x1000);
  }

  .debug_str :
  {
    *(.debug_str)
    . = ALIGN(0x1000);
  }

  .debug_pubnames :
  {
    *(.debug_pubnames)
    . = ALIGN(0x1000);
  }

  .debug_pubtypes :
  {
    *(.debug_pubtypes)
    . = ALIGN(0x1000);
  }

  .debug_frame :
  {
    *(.debug_frame)
    . = ALIGN(0x1000);
  }

  .debug_line :
  {
    *(.debug_line)
    . = ALIGN(0x1000);
  }

  .comment :
  {
    *(.comment)
    . = ALIGN(0x1000);
  }

  .symtab :
  {
    *(.symtab)
    . = ALIGN(0x8);
  }

  .shstrtab :
  {
    *(.shstrtab)
    . = ALIGN(0x1);
  }

  .strtab :
  {
    *(.strtab)
    . = ALIGN(0x1);
  }
}