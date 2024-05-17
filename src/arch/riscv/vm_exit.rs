use crate::arch::TrapContext;
use crate::constants::TRAMPOLINE;
use crate::println;
use crate::sbi::sbi_shutdown;
use riscv::register::mtvec::TrapMode;
use riscv::register::{scause, sepc, sscratch, stval, stvec};

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

#[no_mangle]
pub unsafe fn vm_exit() {
    set_hyp_trap_handler();
    println!("[hypervisor]receive vm exit");
    sbi_shutdown()
}

#[no_mangle]
pub unsafe fn vm_entry(ctx: *mut TrapContext) -> ! {
    set_guest_trap_handler();
    __vm_entry(ctx)
}
