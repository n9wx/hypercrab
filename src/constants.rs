pub const HYPERVISOR_EXTENSION: usize = 0x7;

pub const PAGE_SIZE: usize = 0x1 << 12;
pub const PAGE_SIZE_BITS: usize = 12;
pub const BOOT_STACK_SIZE: usize = PAGE_SIZE * 32;
pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;

pub const MEMORY_END: usize = 0x8020_0000 + 0x100_0000;
