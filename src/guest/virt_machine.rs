use crate::arch::mm::KERNEL_START_PA;
use crate::arch::page_table::PageTableAdapter;
use crate::arch::{vm_exit, TrapContext};
use crate::guest::vcpu::VCpu;
use crate::guest::GuestResource;
use crate::mm::{hpm_guard, AddressSpace, GStagePageTable, GuestAddressSpace, PageTable};
use crate::println;
use alloc::vec::Vec;

pub struct Guest<P: PageTable, G: GStagePageTable> {
    guest_id: usize,
    vcpus: Vec<VCpu>,
    resources: GuestResource<P>,
    address_space: GuestAddressSpace<G>,
}

impl Guest<PageTableAdapter, PageTableAdapter> {
    pub fn new(guest_id: usize, cpu_nums: usize, mem_size: usize) -> Self {
        let mut hpm_guard = hpm_guard();
        let (gpm, host_region) = hpm_guard.alloc_gpm::<PageTableAdapter>(guest_id, mem_size);
        let mut resources = GuestResource::new(host_region);

        let mut vcpus = Vec::with_capacity(cpu_nums);

        // init vcpus context
        for vcpu_id in 0..cpu_nums {
            let stack_region = hpm_guard.alloc_vcpu_stack();
            resources.stack.push(stack_region);
            let context = TrapContext::init_context(
                KERNEL_START_PA,
                resources.hart_stack_top(vcpu_id),
                gpm.token(),
                vm_exit as usize,
            );
            vcpus.push(VCpu::new(context));
        }

        Self {
            guest_id,
            vcpus,
            resources,
            address_space: gpm,
        }
    }

    pub fn load_guest_image(&mut self, guest_data: &[u8]) {
        unsafe {
            let dst = core::slice::from_raw_parts_mut(
                self.resources.get_start_ptr(),
                self.resources.get_mem_size(),
            );

            dst[..guest_data.len()].copy_from_slice(guest_data)
        }
    }

    pub fn vcpu_ctx_ptr(&mut self, vcpu_id: usize) -> *mut TrapContext {
        self.vcpus[vcpu_id].get_ctx_ptr()
    }

    #[inline(always)]
    pub fn get_id(&self) -> usize {
        self.guest_id
    }
}
