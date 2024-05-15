mod frame_allocator;
mod heap_allocator;
mod page_table;
mod vm_space;

use crate::arch::page_table::PageTableAdapter;
use crate::mm::frame_allocator::init_frame_allocator;
use crate::mm::heap_allocator::init_heap;
pub use frame_allocator::{frame_alloc, n_frames_alloc, FrameTracker};
pub use page_table::{GStagePageTable, PageTable};
use spin::{Mutex, MutexGuard, Once};
pub use vm_space::{
    AddressSpace, GuestAddressSpace, HostAddressSpace, MapPermission, MapType, MemRegion,
};

pub fn mm_init() {
    init_frame_allocator();
    init_heap();
    init_address_space();
}

pub static mut HOST_ADDRESS_SPACE: Once<Mutex<HostAddressSpace<PageTableAdapter>>> = Once::new();

fn init_address_space() {
    unsafe {
        HOST_ADDRESS_SPACE.call_once(|| Mutex::new(HostAddressSpace::new_host_space()));
    }
    hpm_guard().activate();
}

pub fn hpm_guard() -> MutexGuard<'static, HostAddressSpace<PageTableAdapter>> {
    unsafe { HOST_ADDRESS_SPACE.get().unwrap().lock() }
}
