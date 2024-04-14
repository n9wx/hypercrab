#![feature(panic_info_message, alloc_error_handler, naked_functions)]
#![no_std]
#![no_main]

use crate::constants::BOOT_STACK_SIZE;
use core::arch::global_asm;
mod arch;
mod console;
mod constants;
mod guest;
mod lang_items;
mod mm;
mod sbi;
mod schedule;

extern crate alloc;

#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("arch/riscv/entry.S"));

#[no_mangle]
pub fn hypervisor_entry(hart_id: usize, dtb_paddress: usize) {
    if arch::is_cup_support_virtualization() {
        println!("current cpu support hardware virtualization!");
        before_start_check();
    }
    walk_fdt(dtb_paddress);
    sbi::sbi_shutdown()
}

pub fn before_start_check() {
    #[cfg(target_arch = "riscv64")]
    {}
}

pub fn walk_fdt(address: usize) {
    let mut fdt = unsafe { fdt::Fdt::from_ptr(address as *const u8).unwrap() };
    for node in fdt.all_nodes() {
        println!("[INFO] find device node {:?}", node.name);
    }
}
