//! constants , structures and functions for Sv39 page based virtual address space

use crate::arch::page_table::{PTEFlags, PageTableEntry, PhysPageNum, VirtPageNum, SECOND_STAGE_PAGE_TABLE_PAGE_NUMS, StepByOne};
use crate::constants::PAGE_SIZE_BITS;
use crate::mm::{frame_alloc, n_frames_alloc, FrameTracker, GStagePageTable, PageTable};
use alloc::vec;
use alloc::vec::Vec;

pub const SV39_PA_WIDTH_BITS: usize = 56;
pub const SV39_PPN_WIDTH_BITS: usize = SV39_PA_WIDTH_BITS - PAGE_SIZE_BITS;
pub const SV39_VA_WIDTH_BITS: usize = 39;
pub const SV39_VPN_WIDTH_BITS: usize = SV39_VA_WIDTH_BITS - PAGE_SIZE_BITS;
pub const PAGE_TRANSLATION_LEVELS: usize = 3;

/// hypervisor space mapped in pa + offset
pub const SV39_KERNEL_SPACE_OFFSET: usize = 0xffff_ffc0_0000_0000;

// vpn base addr = ppn base addr + kernel offset
#[derive(Clone)]
pub struct PageTableAdapter {
    pub root_ppn: PhysPageNum,
    pub frames: Vec<FrameTracker>,
}

impl PageTable for PageTableAdapter {
    fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    fn new_from_kernel() -> Self {
        let root_ppn = {
            use riscv::register::satp;
            satp::read().ppn().into()
        };
        Self {
            root_ppn,
            frames: vec![],
        }
    }

    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, pte_flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, pte_flags | PTEFlags::V);
    }

    fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is unmapped before unmapping", pte);
        *pte = PageTableEntry::invalid();
    }

    fn translate(&mut self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    fn page_table_walk(
        &self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
    ) -> RiscvPageTableWalkIter {
        RiscvPageTableWalkIter::new(start_vpn, end_vpn, self)
    }

    /// return satp regs value
    fn token(&self) -> usize {
        8 << 60 | self.root_ppn.0
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut curr_ppn = self.root_ppn;

        for (i, idx) in indexes.iter().enumerate() {
            let pte = &mut curr_ppn.get_pte_array()[*idx];
            if i == 2 {
                return Some(pte);
            }
            if !pte.is_valid() {
                return None;
            }
            curr_ppn = pte.ppn();
        }
        None
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut curr_ppn = self.root_ppn;

        for (i, idx) in indexes.iter().enumerate() {
            let pte = &mut curr_ppn.get_pte_array()[*idx];
            // todo 如果这个页是刚分配的，pte的内容应该是混乱的，有没有可能刚好v bit是1？
            if i == 2 {
                return Some(pte);
            }
            if !pte.is_valid() {
                let frame = frame_alloc().expect("[kernel] oom!");
                // rwx must be 0 ,refer this is not a leaf pte
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame)
            }
            curr_ppn = pte.ppn();
        }
        None
    }
}

impl GStagePageTable for PageTableAdapter {
    fn new_guest_stage() -> Self {
        let frames = n_frames_alloc(SECOND_STAGE_PAGE_TABLE_PAGE_NUMS).unwrap();
        Self {
            root_ppn: frames[0].ppn,
            frames,
        }
    }
}

pub struct RiscvPageTableWalkIter<'a> {
    current: VirtPageNum,
    end: VirtPageNum,
    page_table: &'a PageTableAdapter,
}

impl<'a> RiscvPageTableWalkIter<'a> {
    pub fn new(
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
        page_table: &'a PageTableAdapter,
    ) -> Self {
        Self {
            current: start_vpn,
            end: end_vpn,
            page_table,
        }
    }
}

impl<'a> Iterator for RiscvPageTableWalkIter<'a> {
    type Item = &'a mut PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.0 != self.end.0 {
            let ret = self.page_table.find_pte(self.current);
            self.current.step();
            ret
        } else {
            None
        }
    }
}
