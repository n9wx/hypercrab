use crate::println;
use crate::sbi::sbi_shutdown;
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec;
use crate::arch::TrapContext; 

extern "C" {
    pub fn __vm_exit();
    pub fn __vm_entry(context: *mut TrapContext);
}

pub fn trap_from_hyp() {
    println!("[Hypervisor] trap from H-S mode!");
    sbi_shutdown();
}

pub fn set_hyp_trap_handler() {
    let __hyp_trap_handler = trap_from_hyp as usize;
    unsafe {
        stvec::write(__hyp_trap_handler, TrapMode::Direct);
    }
}

#[no_mangle]
pub unsafe fn vm_exit() {
    set_hyp_trap_handler();
    println!("[hypervisor]receive vm exit")
}
