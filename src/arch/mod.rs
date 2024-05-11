#[cfg(target_arch = "riscv64")]
mod riscv;

#[cfg(target_arch = "riscv64")]
pub use riscv::*;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub fn is_cup_support_virtualization() -> bool {
    #[cfg(target_arch = "riscv64")]
    riscv::is_cpu_support()
}
