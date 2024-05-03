mod frame_allocator;
mod heap_allocator;
mod page_table;
mod vm_space;

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::page_table::*;

pub use frame_allocator::{frame_alloc, frame_dealloc, n_frames_alloc, FrameTracker};
pub use page_table::{GStagePageTable, PageTable};
