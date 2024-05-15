.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(a0)
.endm

    .section .text.trampoline
    .global __vm_exit
    .global __vm_entry

    .align 2
__vm_exit:
    # 交换sp和sscratch,
    # sp -> TrapContext sscratch -> hyp_sp
    csrrw sp,sscratch,sp
    sd x1,1*8(sp)
    sd x3,3*8(sp)
    # it's tp ,maybe we can store hartid or something else on it
    sd x4,4*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # save guest address space sp
    csrr t1,sscratch
    sd t1,2*8(sp)

    # now we can use gp freely
    # store sstatus
    csrr t0,sstatus
    sd t0,32*8(sp)
    # store sepc
    csrr t0,sepc
    sd t0,33*8(sp)
    csrr t0,hstatus
    sd t0,34*8(sp)
    csrr t0,hgatp
    sd t0,35*8(sp)
    #  load trap_handler
    ld t1,37*8(sp)
    # set stack ptr in hypervisor address space
    ld sp,36*8(sp)
    # now,jump to hypervisor world !
    jr t1

# __vm_entry(*mut TrapContext)
__vm_entry:
    # first record the ptr of current vcpu context
    csrw sscratch,a0
    # restore hs level csr first
    ld t0,33*8(sp)
    csrw sstatus,t0
    ld t1,35*8(sp)
    csrw hgatp,t1
    ld t0,34*8(a0)
    # load hgatp and flush Gstage pagetable
    csrw hstatus,t0
    hfence.gvma
    ld t1,32*8(a0)
    csrw sepc,t1

    .set n,1
    .rept 9
        LOAD_GP %n
        .set n,n+1
    .endr
    #resotre a0 at last
    .set n,11
    .rept 21
        LOAD_GP %n
        .set n,n+1
    .endr
    # restore a0 here
    ld a0,10*8(a0)
    sret
