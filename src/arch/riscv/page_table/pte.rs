use bitflags::*;
use crate::arch::riscv::page_table::address::PhysPageNum;
use crate::arch::riscv::page_table::sv39::SV39_PPN_WIDTH_BITS;

bitflags! {
    /// page table entry flags for riscv
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct PageTableEntry {
    pub entry: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self { entry: ppn.0 << 10 | flags.bits as usize }
    }

    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.entry >> 10 & ((1 << SV39_PPN_WIDTH_BITS) - 1))
    }

    #[inline(always)]
    fn flags(&self) -> u8 {
        (self.entry & 0xff) as u8
    }

    pub fn is_valid(&self) -> bool {
        self.flags() & PTEFlags::V.bits != 0
    }

    pub fn readable(&self) -> bool {
        self.flags() & PTEFlags::R.bits != 0
    }

    pub fn writable(&self) -> bool {
        self.flags() & PTEFlags::W.bits != 0
    }

    pub fn executable(&self) -> bool {
        self.flags() & PTEFlags::X.bits != 0
    }

    pub fn is_user(&self) -> bool {
        self.flags() & PTEFlags::U.bits != 0
    }

    pub fn is_global(&self) -> bool {
        self.flags() & PTEFlags::G.bits != 0
    }

    pub fn is_accessed(&self) -> bool {
        self.flags() & PTEFlags::A.bits != 0
    }

    pub fn is_dirty(&self) -> bool {
        self.flags() & PTEFlags::D.bits != 0
    }
}