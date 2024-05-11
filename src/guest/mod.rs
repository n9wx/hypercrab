mod vcpu;
mod virt_machine;

use crate::arch::page_table::PageTableAdapter;
use crate::mm::{MemRegion, PageTable};
use alloc::vec::Vec;
pub use virt_machine::VirtMachine;

// virt machine = gpa address space + device + vcpus
// guest = virt machine + resource(mem region(region represent gpm space)+stack for each vcpu ) in host machine

/// struct represent mem resource used by guest
pub struct GuestResource<P: PageTable> {
    pub normal_mem: MemRegion<P>,
    pub stack: Vec<MemRegion<P>>,
}

impl<P: PageTable> GuestResource<P> {
    pub fn new(mem: MemRegion<P>) -> Self {
        Self {
            normal_mem: mem,
            stack: Vec::new(),
        }
    }

    pub fn hart_stack_top(&self, vcpu_id: usize) -> usize {
        assert!(
            self.stack.len() > vcpu_id,
            "[GuestResource] guest has no vcpu:{vcpu_id}"
        );
        self.stack[vcpu_id].start_vpn.into()
    }
}
