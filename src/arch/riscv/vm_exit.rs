use crate::arch::TrapContext;
use crate::constants::TRAMPOLINE;
use crate::println;
use crate::sbi::sbi_shutdown;
use riscv::register::mtvec::TrapMode;
use riscv::register::scause::{Exception, Trap};
use riscv::register::{htinst, htval, scause, sepc, sscratch, stval, stvec, vsatp};

extern "C" {
    pub fn __vm_exit();
    pub fn __vm_entry(context: *mut TrapContext) -> !;

    pub fn __traps_in_hyp();
}

pub fn trap_from_hyp() {
    println!(
        "[Hypervisor] trap from H-S mode! scause:{:?} stval:{:x}",
        scause::read().cause(),
        stval::read() // 1
    );
    sbi_shutdown();
}

pub fn set_hyp_trap_handler() {
    let trap_va = __traps_in_hyp as usize - __vm_exit as usize + TRAMPOLINE;
    unsafe {
        stvec::write(trap_va, TrapMode::Direct);
        sscratch::write(trap_from_hyp as usize);
    }
}

#[inline(always)]
pub fn set_guest_trap_handler() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

/// handle trap from V mode(VS or VU?)
#[no_mangle]
pub unsafe fn vm_exit() {
    set_hyp_trap_handler();
    let scause = scause::read().cause();
    match scause {
        Trap::Interrupt(_) => {}
        Trap::Exception(Exception::InstructionGuestPageFault) => {
            let stval = stval::read();
            let htval = htval::read();
            let sepc = sepc::read();
            println!(
                "[hypervisor]receive vm exit scause:{:?} spec:{:#x} stval:{:#x} htval:{:#x} ",
                scause, sepc, stval, htval
            );
        }
        _ => (),
    }
    let stval = stval::read();
    let htval = htval::read();
    let sepc = sepc::read();
    println!(
        "[hypervisor]receive vm exit scause:{:?} spec:{:#x} stval:{:#x} htval:{:#x} vsatp:  {:#x} ",
        scause,
        sepc,
        stval,
        htval << 2,
        vsatp::read().bits()
    );

    sbi_shutdown()
}

#[no_mangle]
pub unsafe fn vm_entry(ctx: *mut TrapContext) -> ! {
    set_guest_trap_handler();
    __vm_entry(ctx)
}
