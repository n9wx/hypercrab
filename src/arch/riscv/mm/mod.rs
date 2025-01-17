pub use RISCV_GUEST_START_VA as GUEST_START_VA;
pub use RISCV_KERNEL_START_PA as KERNEL_START_PA;

// vm space start at 1G 
pub const RISCV_GUEST_START_VA: usize = 0x1_0000_0000;

pub const RISCV_KERNEL_START_PA: usize = 0x8020_0000;
