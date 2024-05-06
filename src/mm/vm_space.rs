use crate::arch::page_table::PTEFlags;
use crate::constants::MEMORY_END;
use crate::mm::frame_allocator::FrameTracker;
use crate::mm::page_table::{
    active_page_table, fill_guest_page_table, CombinedWalker, PPNRange, PageTableAdapter,
    PhysAddress, PhysPageNum, VPNRange, VirtAddress, VirtPageNum,
};
use crate::mm::{frame_alloc, GStagePageTable, PageTable, KERNEL_START_PA};
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
        let end_vpn = VirtAddress(start_va.0 + size).current_page_number();
        Self {
            start_vpn,
            page_nums: end_vpn.0 - start_vpn.0 + 1,
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
                self.data_frames.insert(vpn, frame_tracker.clone());
                frame_tracker.ppn
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
    pub fn end_vpn(&self) -> VirtPageNum {
        VirtPageNum(self.start_vpn.0 + self.page_nums + 1)
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
}

pub trait AddressSpace<P: PageTable> {
    type VirtAddress;
    type PhysAddress;
    fn translate_va(&self, va: Self::VirtAddress) -> Option<Self::PhysAddress>;
    fn map_region(&mut self, vm_region: MemRegion<P>);
}

pub struct HostAddressSpace<P: PageTable> {
    regions: Vec<MemRegion<P>>,
    page_table: P,
}

/// guest  address space descriptor,represent as a host address region
pub struct GuestAddressSpace<S: GStagePageTable> {
    regions: Vec<MemRegion<S>>,
    page_table: S,
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
}

impl<P: PageTable> HostAddressSpace<P> {
    fn new_bare() -> Self {
        Self {
            regions: Vec::new(),
            page_table: P::new(),
        }
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
            MapPermission::R | MapPermission::X,
        ));

        host_vm_space
    }

    pub fn alloc_vm_space<G: GStagePageTable>(
        &mut self,
        start_va: VirtAddress,
        size: usize,
    ) -> GuestAddressSpace<G> {
        // first map guest mem region to host address space
        let mut host_map_region = MemRegion::<P>::new(
            start_va,
            size,
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        );
        // don't push mem region into host address space now
        host_map_region.map(&mut self.page_table);

        // host mem region
        let host_start_vpn = start_va.current_page_number();
        let page_nums =
            VirtAddress(start_va.0 + size).current_page_number().0 - host_start_vpn.0 + 1;
        let host_end_vpn = host_map_region.end_vpn();

        // setup guest pm space
        let mut guest_pm_space = GuestAddressSpace::<G>::new_bare();
        let guest_start_ppn = PhysPageNum::from(PhysAddress(KERNEL_START_PA));
        let guest_end_ppn = PhysPageNum(guest_start_ppn.0 + page_nums);

        // todo 因为现在的FrameTracker实现了drop又没有做引用计数，所以暂时用没有携带任何页面的mem region 填充 guest pm space
        let guest_mem_region = MemRegion::<G>::new(
            VirtAddress(KERNEL_START_PA),
            size,
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::X,
        );
        guest_pm_space.regions.push(guest_mem_region);

        // fill g stage page table for guest
        let combined_walker = CombinedWalker::new(
            &self.page_table,
            &mut guest_pm_space.page_table,
            VPNRange::new(host_start_vpn, host_end_vpn),
            PPNRange::new(guest_start_ppn, guest_end_ppn),
        );
        fill_guest_page_table(combined_walker);

        guest_pm_space
    }

    /// active page based virtual address space
    pub fn active(&self) {
        let token = self.page_table.token();
        unsafe {
            active_page_table(token);
        }
    }
}

impl<S: GStagePageTable> GuestAddressSpace<S> {
    pub fn new_bare() -> Self {
        Self {
            regions: vec![],
            page_table: S::new_guest_stage(),
        }
    }

    // pub fn add_gpa_region(&mut self,gpa_range:VPNRange)
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
}
