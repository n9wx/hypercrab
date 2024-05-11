use crate::arch::mm::KERNEL_START_PA;
use crate::arch::page_table::PageTableAdapter;
use crate::arch::{self, vm_exit, TrapContext};
use crate::constants::GUEST_STACK_SIZE;
use crate::guest::vcpu::VCpu;
use crate::guest::GuestResource;
use crate::hypervisor::{get_host_guard, HOST_MACHINE};
use crate::mm::{AddressSpace, GStagePageTable, GuestAddressSpace, PageTable};
use alloc::vec::Vec;

pub struct VirtMachine<const N: usize, G: GStagePageTable> {
    vcpus: [VCpu; N],
    address_space: GuestAddressSpace<G>,
}

impl<const N: usize, G: GStagePageTable> VirtMachine<N, G> {
    /*    pub fn new(address_space: GuestAddressSpace<G>) -> Self {
        // todo set guest stack in hyp address space
        let context = TrapContext::init_context(
            arch::mm::KERNEL_START_PA,
            HypervisorStack::alloc_hstack(0).0,
            address_space.token(),
            arch::vm_exit as usize,
        );
        Self {
            vcpus: [VCpu::new(context); N],
            address_space,
        }
    }*/
}

pub struct Guest<P: PageTable, G: GStagePageTable> {
    guest_id: usize,
    vcpus: Vec<VCpu>,
    resources: GuestResource<P>,
    address_space: GuestAddressSpace<G>,
}

impl Guest<PageTableAdapter, PageTableAdapter> {
    pub fn new(guest_id: usize, cpu_nums: usize) -> Self {
        let mut host_guard = get_host_guard();
        let (gpm, host_region) = host_guard
            .address_space
            .alloc_gpm::<PageTableAdapter>(guest_id, GUEST_STACK_SIZE);
        let mut resources = GuestResource::new(host_region);

        let mut vcpus = Vec::with_capacity(cpu_nums);

        // init vcpus context
        for vcpu_id in 0..cpu_nums {
            let stack_region = host_guard.address_space.alloc_vcpu_stack();
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
}
