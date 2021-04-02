/* src/arch/x86_64/linker.ld */

ENTRY(_start)

SECTIONS {
  . = 0x200000;

  .rodata :
  {
    *(.rodata)
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
    *(.text)
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
    *(.data)
    . = ALIGN(0x1000);
  }

  .bss :
  {
    *(.bss)
    . = ALIGN(0x1000);
  }


  .comment :
  {
    *(.comment)
    . = ALIGN(0x1000);
  }

}