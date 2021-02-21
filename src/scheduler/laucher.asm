    global launch_asm

    section .text
launch_asm:
     mov ax,0x23
     mov ds,ax
     mov es,ax 
     mov fs,ax 
     mov gs,ax ;we don't need to worry about SS. it's handled by iret
 
     mov eax,esp
     pushq 0x23 ;user data segment with bottom 2 bits set for ring 3
     pushq rsi ;push our current esp for the iret stack frame
     pushf
     pushq 0x1B; ;user code segment with bottom 2 bits set for ring 3
     pushq rdi
     iretq