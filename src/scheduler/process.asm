    global leave_context

leave_context:
    mov rsp, rdi
    pop rbx
    pop rcx
    pop rbp
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    pop r9
    pop r8
    pop r10
    pop rdx
    pop rsi
    pop rdi
    pop rax
    sti
    iretq