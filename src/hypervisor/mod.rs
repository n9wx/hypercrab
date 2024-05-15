use crate::arch::page_table::PageTableAdapter;
use crate::arch::{vm_entry, TrapContext};
use crate::guest::Guest;
use alloc::collections::LinkedList;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Mutex, MutexGuard, Once};

pub static mut GUESTS_QUEUE: Once<Mutex<LinkedList<Guest<PageTableAdapter, PageTableAdapter>>>> =
    Once::new();
pub static GUEST_ID: AtomicUsize = AtomicUsize::new(0);

#[inline(always)]
pub fn alloc_guest_id() -> usize {
    GUEST_ID.fetch_add(1, Ordering::SeqCst)
}
pub fn init_guest_queue() {
    unsafe {
        GUESTS_QUEUE.call_once(|| Mutex::new(LinkedList::new()));
    }
}

pub fn queue_guard() -> MutexGuard<'static, LinkedList<Guest<PageTableAdapter, PageTableAdapter>>> {
    unsafe { GUESTS_QUEUE.get().unwrap().lock() }
}

/// create guest and add guest to queue
pub fn create_guest(cpu_nums: usize, mem_size: usize, guest_data: &[u8]) -> usize {
    let guest_id = alloc_guest_id();
    let mut guest = Guest::new(guest_id, cpu_nums, mem_size);
    guest.load_guest_image(guest_data);
    queue_guard().push_back(guest);
    guest_id
}

pub fn run_guest(guest_id: usize) -> ! {
    let mut ctx: *mut TrapContext = core::ptr::null_mut();
    let mut queue_guard = queue_guard();
    for guest in queue_guard.iter_mut() {
        if guest.get_id() == guest_id {
            ctx = guest.vcpu_ctx_ptr(0);
            break;
        }
    }
    drop(queue_guard);

    unsafe { vm_entry(ctx) }
}
