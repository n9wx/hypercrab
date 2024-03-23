#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "riscv64")]
pub mod riscv;

pub fn is_cup_support_virtualization() -> bool {
    #[cfg(target_arch = "riscv64")]
    riscv::is_cpu_support()
}