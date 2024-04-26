//! constants , structures and functions for Sv39 page based virtual address space

use crate::arch::page_table::{
    PTEFlags, PageTableEntry, PhysPageNum, VirtPageNum, SECOND_STAGE_PAGE_TABLE_PAGE_NUMS,
};
use crate::constants::PAGE_SIZE_BITS;
use crate::mm::{frame_alloc, n_frames_alloc, FrameTracker, PageTable, SecondStagePageTable};
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

impl PageTableAdapter {
    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
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

    pub fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
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
        }
        None
    }
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

    /// return satp regs value
    fn token(&self) -> usize {
        8 << 60 | self.root_ppn.0
    }
}

impl SecondStagePageTable for PageTableAdapter {
    fn new_second_stage() -> Self {
        let frames = n_frames_alloc(SECOND_STAGE_PAGE_TABLE_PAGE_NUMS).unwrap();
        Self {
            root_ppn: frames[0].ppn,
            frames,
        }
    }
}
