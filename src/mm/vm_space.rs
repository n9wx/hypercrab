use crate::arch::mm::{GUEST_START_VA, KERNEL_START_PA};
use crate::arch::page_table::{
    active_page_table, PPNRange, PTEFlags, PhysAddress, PhysPageNum, VPNRange, VirtAddress,
    VirtPageNum,
};
use crate::constants::{GUEST_STACK_SIZE, GUEST_STACK_TOP, MEMORY_END, PAGE_SIZE, TRAMPOLINE};
use crate::mm::frame_allocator::FrameTracker;
use crate::mm::page_table::{fill_guest_page_table, CombinedWalker};
use crate::mm::{frame_alloc, GStagePageTable, PageTable};
use crate::GUEST_IMAGE;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::marker::PhantomData;

bitflags! {
    pub struct MapPermission:u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MapType {
    Linear(PhysPageNum),
    Framed,
}

impl MapType {
    #[inline(always)]
    pub fn new_linear(pa: PhysAddress) -> Self {
        Self::Linear(pa.current_page_number())
    }
}

/// represent a contiguous piece of virtual memory
pub struct MemRegion<P: PageTable> {
    pub start_vpn: VirtPageNum, //must be page boundary align
    pub page_nums: usize,
    pub map_type: MapType,
    pub data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    pub permission: MapPermission,
    _marker: PhantomData<P>,
}

impl<P: PageTable> MemRegion<P> {
    pub fn new(
        start_va: VirtAddress,
        size: usize,
        map_type: MapType,
        permission: MapPermission,
    ) -> Self {
        let start_vpn = start_va.current_page_number();
        let end_vpn = VirtAddress(start_va.0 + size).next_page_number();
        Self {
            start_vpn,
            page_nums: end_vpn.0 - start_vpn.0,
            map_type,
            data_frames: BTreeMap::new(),
            permission,
            _marker: PhantomData,
        }
    }
    /*    pub fn new(
        start_va: VirtAddress,
        end_va: VirtAddress,
        permission: MapPermission,
        type_: MapType,
    ) -> Self {
        let start_vpn = start_va.next_page_number();
        let end_vpn = end_va.current_page_number();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            ppn_range: None,
            data_frames: Default::default(),
            permission,
            map_type: type_,
            _marker: PhantomData,
        }
    }*/

    pub fn map_one(&mut self, page_table: &mut P, vpn: VirtPageNum) {
        let ppn = match self.map_type {
            MapType::Linear(start_ppn) => {
                let offset = vpn.0 - self.start_vpn.0;
                PhysPageNum(start_ppn.0 + offset)
            }
            MapType::Framed => {
                let frame_tracker = frame_alloc().unwrap();
                let ppn = frame_tracker.ppn;
                self.data_frames.insert(vpn, frame_tracker);
                ppn
            }
        };
        let pte_flags = PTEFlags::from_bits(self.permission.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    pub fn unmap_one(&mut self, page_table: &mut P, vpn: VirtPageNum) {
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }

    pub fn map(&mut self, page_table: &mut P) {
        for offset in 0..self.page_nums {
            let vpn = (self.start_vpn.0 + offset).into();
            self.map_one(page_table, vpn);
        }
    }

    pub fn unmap(&mut self, page_table: &mut P) {
        for offset in 0..self.page_nums {
            page_table.unmap((self.start_vpn.0 + offset).into());
        }
    }

    #[inline]
    pub fn start_vpn(&self) -> VirtPageNum {
        self.start_vpn
    }

    #[inline]
    pub fn end_vpn(&self) -> VirtPageNum {
        VirtPageNum(self.start_vpn.0 + self.page_nums)
    }

    pub fn get_size(&self) -> usize {
        self.page_nums * PAGE_SIZE
    }
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

pub trait AddressSpace<P: PageTable> {
    type VirtAddress;
    type PhysAddress;
    fn translate_va(&self, va: Self::VirtAddress) -> Option<Self::PhysAddress>;
    fn map_region(&mut self, vm_region: MemRegion<P>);
    fn token(&self) -> usize;
}

pub struct HostAddressSpace<P: PageTable> {
    regions: Vec<MemRegion<P>>, //host mem_regions
    gpm_base: usize,
    vcpu_stack_base: usize,
    page_table: P,
}

/// guest  address space descriptor,represent as a host address region
pub struct GuestAddressSpace<G: GStagePageTable> {
    pub guest_id: usize,
    pub regions: Vec<MemRegion<G>>,
    pub page_table: G,
}

impl<P: PageTable> AddressSpace<P> for HostAddressSpace<P> {
    type VirtAddress = VirtAddress;
    type PhysAddress = PhysAddress;

    fn translate_va(&self, va: Self::VirtAddress) -> Option<Self::PhysAddress> {
        todo!()
    }

    fn map_region(&mut self, mut vm_region: MemRegion<P>) {
        vm_region.map(&mut self.page_table);
        self.regions.push(vm_region);
    }

    fn token(&self) -> usize {
        self.page_table.token()
    }
}

impl<P: PageTable> HostAddressSpace<P> {
    fn new_bare() -> Self {
        Self {
            regions: Vec::new(),
            gpm_base: GUEST_START_VA,
            vcpu_stack_base: GUEST_STACK_TOP - GUEST_STACK_SIZE,
            page_table: P::new(),
        }
    }

    pub fn map_embedded_guest(&mut self) {
        let start_address = GUEST_IMAGE.as_ptr() as usize;
        let mut guest_image_region = MemRegion::<P>::new(
            start_address.into(),
            GUEST_IMAGE.len(),
            MapType::new_linear(start_address.into()),
            MapPermission::R | MapPermission::W | MapPermission::X,
        );
        guest_image_region.map(&mut self.page_table);
        self.regions.push(guest_image_region);
    }

    pub fn new_host_space() -> Self {
        let mut host_vm_space = Self::new_bare();

        // map kernel .text section
        host_vm_space.map_region(MemRegion::new(
            (stext as usize).into(),
            (etext as usize) - (stext as usize),
            MapType::new_linear((stext as usize).into()),
            MapPermission::R | MapPermission::X,
        ));

        // map .rodata
        host_vm_space.map_region(MemRegion::new(
            (srodata as usize).into(),
            (erodata as usize) - (srodata as usize),
            MapType::new_linear((srodata as usize).into()),
            MapPermission::R,
        ));

        // map .data section
        host_vm_space.map_region(MemRegion::new(
            (sdata as usize).into(),
            (edata as usize) - (sdata as usize),
            MapType::new_linear((sdata as usize).into()),
            MapPermission::R | MapPermission::W,
        ));

        // map .bss section
        host_vm_space.map_region(MemRegion::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize) - (sbss_with_stack as usize),
            MapType::new_linear((sbss_with_stack as usize).into()),
            MapPermission::R | MapPermission::W,
        ));

        // identical map physics frame to vmm address space
        host_vm_space.map_region(MemRegion::new(
            (ekernel as usize).into(),
            MEMORY_END - (ekernel as usize),
            MapType::new_linear((ekernel as usize).into()),
            MapPermission::R | MapPermission::W,
        ));

        // map trampoline for hypervisor
        host_vm_space.page_table.map(
            VirtAddress(TRAMPOLINE).into(),
            PhysAddress(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );

        host_vm_space.map_embedded_guest();

        host_vm_space
    }

    pub fn alloc_gpm<G: GStagePageTable>(
        &mut self,
        guest_id: usize,
        size: usize,
    ) -> (GuestAddressSpace<G>, MemRegion<P>) {
        let mut host_map_region = MemRegion::<P>::new(
            self.gpm_base.into(),
            size,
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        );
        host_map_region.map(&mut self.page_table);

        // update gpm start va
        self.gpm_base = PAGE_SIZE + host_map_region.end_vpn().page_base_va().0;

        let host_start_vpn = host_map_region.start_vpn();
        let host_end_vpn = host_map_region.end_vpn();
        let page_nums = host_map_region.page_nums;

        let mut gpm = GuestAddressSpace::<G>::new_bare(guest_id);
        let guest_start_ppn = PhysPageNum::from(PhysAddress(KERNEL_START_PA));
        let guest_end_ppn = PhysPageNum(guest_start_ppn.0 + page_nums);

        // todo 因为现在的FrameTracker实现了drop又没有做引用计数，所以暂时用没有携带任何页面的mem region 填充 guest pm space
        let guest_mem_region = MemRegion::<G>::new(
            VirtAddress(KERNEL_START_PA),
            size,
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::X,
        );

        // fill g stage page table for guest
        let combined_walker = CombinedWalker::new(
            &self.page_table,
            &mut gpm.page_table,
            VPNRange::new(host_start_vpn, host_end_vpn),
            PPNRange::new(guest_start_ppn, guest_end_ppn),
        );
        fill_guest_page_table(combined_walker);
        gpm.regions.push(guest_mem_region);

        (gpm, host_map_region)
    }

    /// alloc stack regions and map to hyp address space
    pub fn alloc_vcpu_stack(&mut self) -> MemRegion<P> {
        let mut stack_region = MemRegion::<P>::new(
            self.vcpu_stack_base.into(),
            GUEST_STACK_SIZE,
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        );
        stack_region.map(&mut self.page_table);
        self.vcpu_stack_base -= GUEST_STACK_SIZE + PAGE_SIZE;

        stack_region
    }

    /// active page based virtual address space
    pub fn activate(&self) {
        let token = self.page_table.token();
        unsafe {
            active_page_table(token);
        }
    }
}

impl<G: GStagePageTable> GuestAddressSpace<G> {
    pub fn new_bare(guest_id: usize) -> Self {
        Self {
            guest_id,
            regions: vec![],
            page_table: G::new_guest_stage(),
        }
    }
}

impl<S: GStagePageTable> AddressSpace<S> for GuestAddressSpace<S> {
    type VirtAddress = ();
    type PhysAddress = ();

    fn translate_va(&self, va: Self::VirtAddress) -> Option<Self::PhysAddress> {
        todo!()
    }

    fn map_region(&mut self, mut vm_region: MemRegion<S>) {
        vm_region.map(&mut self.page_table);
        self.regions.push(vm_region);
    }

    fn token(&self) -> usize {
        self.page_table.token()
    }
}

pub fn gpa2hva(guest_id: usize, gpa: usize) -> usize {
    todo!()
}
