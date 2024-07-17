pub mod context;
pub mod mm;
pub mod page_table;
pub mod vm_exit;
mod intc;

pub use context::*;
use core::arch::global_asm;
pub use vm_exit::*;

#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("trap.S"));

pub fn is_cpu_support() -> bool {
    use crate::constants::HYPERVISOR_EXTENSION;
    use crate::sbi::sbi_probe_extension;
    use crate::sbi::SBI_SUCCESS;

    sbi_probe_extension(HYPERVISOR_EXTENSION) == SBI_SUCCESS
}
