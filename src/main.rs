#![feature(panic_info_message)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

mod arch;
mod sbi;
mod constants;
mod lang_items;
mod console;


#[link_section = ".bss.stack"]
static BOOT_STACK: [u8; (1 << 12) * 32] = [0u8; (1 << 12) * 32];

#[link_section = ".text.entry"]
#[export_name = "_start"]
#[naked]
pub unsafe extern "C" fn start() -> ! {
    use core::arch::asm;
    asm!(
    "la sp,{boot_stack}",
    "call hypervisor_entry",
    boot_stack = sym BOOT_STACK,
    options(noreturn)
    )
}

#[no_mangle]
pub fn hypervisor_entry(hart_id: usize) {
    if arch::is_cup_support_virtualization() {
        println!("current cpu support hardware virtualization!")
    }
    sbi::sbi_shutdown()
}