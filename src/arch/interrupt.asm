int_handle:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov rdi, [rsp + 136]
    mov rsi, [rsp + 128]
    mov rdx, (rsp + 144)
    mov rax, [rsp + 128]
    imul rax, 8
    add rax, handlers
    mov rax, [rax]
    cmp rax, 0x00
    cld
    je no_handler
    call rax
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    add rsp, 16
    iretq

handlers:
    .fill 2048, 1, 0x00
