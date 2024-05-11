use crate::arch::page_table::PageTableAdapter;
use crate::guest::VirtMachine;
use crate::mm::{GStagePageTable, HostAddressSpace, PageTable};
use alloc::vec;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard, Once};

pub static mut HOST_MACHINE: Once<Mutex<HostMachine<PageTableAdapter, PageTableAdapter>>> =
    Once::new();

pub fn init_host(host_address_space: HostAddressSpace<PageTableAdapter>) {
    unsafe {
        HOST_MACHINE.call_once(|| Mutex::new(HostMachine::init_hypervisor(host_address_space)));
    }
}

#[inline]
pub fn get_host_guard() -> MutexGuard<'static, HostMachine<PageTableAdapter, PageTableAdapter>> {
    unsafe { HOST_MACHINE.get_mut().unwrap().lock() }
}

pub struct HostMachine<P: PageTable, G: GStagePageTable> {
    pub address_space: HostAddressSpace<P>,
    pub guests: Vec<VirtMachine<1, G>>,
}

impl<P: PageTable, G: GStagePageTable> HostMachine<P, G> {
    pub fn init_hypervisor(address_space: HostAddressSpace<P>) -> Self {
        Self {
            address_space,
            guests: vec![],
        }
    }
}
