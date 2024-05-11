mod frame_allocator;
mod heap_allocator;
mod page_table;
mod vm_space;

pub use frame_allocator::{frame_alloc, n_frames_alloc, FrameTracker};
pub use page_table::{GStagePageTable, PageTable};
pub use vm_space::{
    AddressSpace, GuestAddressSpace, HostAddressSpace, MapPermission, MapType, MemRegion,
};
