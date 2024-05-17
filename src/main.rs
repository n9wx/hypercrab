#![feature(panic_info_message, alloc_error_handler)]
#![no_std]
#![no_main]

use crate::arch::page_table::PageTableAdapter;
use crate::arch::set_hyp_trap_handler;
use crate::constants::GUEST_MEM_SIZE;
use crate::hypervisor::{create_guest, init_guest_queue, run_guest};
use crate::mm::{mm_init, HostAddressSpace};
use core::arch::global_asm;

mod arch;
mod console;
mod constants;
mod guest;
mod hypervisor;
mod lang_items;
mod mm;
mod sbi;
mod schedule;

extern crate alloc;

#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("arch/riscv/entry.S"));

#[link_section = ".initrd"]
static GUEST_IMAGE: [u8; include_bytes!("../guest.bin").len()] = *include_bytes!("../guest.bin");

#[no_mangle]
pub fn hypervisor_entry(hart_id: usize, dtb_paddress: usize) {
    if arch::is_cup_support_virtualization() {
        println!("current cpu support hardware virtualization!");
        // before_start_check();
    }
    // walk_fdt(dtb_paddress);
    mm_init();
    init_guest_queue();
    println!("[hypervisor] init host address space success!");
    set_hyp_trap_handler();
    println!("[hypervisor]set hyp trap handler");
    unsafe {
        let guest_id = create_guest(1, GUEST_MEM_SIZE, &GUEST_IMAGE);
        println!("load guest bin!");
        run_guest(guest_id);
    }

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
