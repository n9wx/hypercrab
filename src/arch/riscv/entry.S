    .section .text.entry
    .global _start
_start:
    # a0 hart id
    # a1 dtb address
    slli t0,a0,16 #stack offset = hart id * stack size (hart_id << 16)
    la sp,boot_stack_top
    sub sp,sp,t0

    # load page table to satp
    # la t0,boot_page_table # 56 bits pa
    # satp = mode(8 << 60) | boot_page_table
    # li t1,8 << 60
    # srli t0,t0,12 #boot page table ppn
    # or t0,t1,t0
    # csrw satp,t0
    # sfence.vma
    call hypervisor_entry

    .section .bss.stack
    .global boot_stack_end
boot_stack_end:
    .space 4096 * 16 * 8 # 16 pages per core

    .global boot_stack_top
boot_stack_top:


# set boot page table here for kernel(hypervisor) virt address space
    .section .data
    .align   12 # align to page boundary
    .global  boot_page_table
boot_page_table:
    # map pa 0x8000_0000 to
    # 1. va 0x8000_0000
    # 2. 0xffff_ffc0_8000_0000
    # so we fill 2 ptes here
    .quad   0
    .quad   0
    .quad   (0x80000 << 10) | 0xcf #ADRWXV
    .zero   8 * 255
    .quad   (0x80000 << 10) | 0xcf #ADRWXV
    .zero   8 * 253
