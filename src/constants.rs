pub const HYPERVISOR_EXTENSION: usize = 0x7;

pub const PAGE_SIZE: usize = 0x1 << 12;
pub const PAGE_SIZE_BITS: usize = 12;
pub const BOOT_STACK_SIZE: usize = PAGE_SIZE * 32;
pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;

pub const MEMORY_END: usize = 0x8020_0000 + 0x100_0000;
pub const CPU_NUMS: usize = 1;

pub const GUEST_STACK_SIZE: usize = PAGE_SIZE * 16;

pub const GUEST_STACK_BASE: usize = TRAMPOLINE - PAGE_SIZE;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
