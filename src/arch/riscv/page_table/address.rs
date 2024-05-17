use crate::arch::riscv::page_table::pte::PageTableEntry;
use crate::arch::riscv::page_table::sv39::{
    PAGE_TRANSLATION_LEVELS, SV39_PA_WIDTH_BITS, SV39_PPN_WIDTH_BITS, SV39_VA_WIDTH_BITS,
    SV39_VPN_WIDTH_BITS,
};
use crate::arch::riscv::page_table::VPN_INDEX_WIDTH_BITS;
use crate::constants::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{Debug, Formatter};

macro_rules! debug_impl {
    ($type:ty,$str:literal) => {
        impl Debug for $type {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}:{:#x}", $str, self.0)
            }
        }
    };
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

debug_impl!(PhysPageNum, "PPN");

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddress(pub usize);

debug_impl!(PhysAddress, "PA");

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

debug_impl!(VirtPageNum, "VPN");

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddress(pub usize);

debug_impl!(VirtAddress, "VA");

macro_rules! from_impl {
    ($type_name:ty,$mask:tt) => {
        impl From<usize> for $type_name {
            fn from(value: usize) -> Self {
                Self(value & ((1 << $mask) - 1))
            }
        }
    };
}

from_impl!(PhysPageNum, SV39_PPN_WIDTH_BITS);
from_impl!(PhysAddress, SV39_PA_WIDTH_BITS);
from_impl!(VirtPageNum, SV39_VPN_WIDTH_BITS);
// from_impl!(VirtAddress, SV39_VA_WIDTH_BITS);

impl From<usize> for VirtAddress {
    fn from(v: usize) -> Self {
        if v >= (1 << (SV39_VA_WIDTH_BITS - 1)) {
            VirtAddress(v | (!((1 << SV39_VA_WIDTH_BITS) - 1)))
        } else {
            VirtAddress(v)
        }
    }
}

macro_rules! into_impl {
    ($type:ty) => {
        impl From<$type> for usize {
            fn from(value: $type) -> Self {
                value.0
            }
        }
    };
}

into_impl!(PhysPageNum);
into_impl!(PhysAddress);
into_impl!(VirtPageNum);

impl From<VirtAddress> for usize {
    fn from(value: VirtAddress) -> Self {
        // if virtual address bigger than 2^38 - 1,bits higher than 38 must all be 1
        if value.0 >= ((1 << (SV39_VA_WIDTH_BITS - 1)) - 1) {
            value.0 | !((1 << (SV39_VA_WIDTH_BITS - 1)) - 1)
        } else {
            value.0
        }
    }
}

macro_rules! address_impl {
    ($type:ty,$page_number_type:tt) => {
        impl $type {
            pub fn current_page_number(&self) -> $page_number_type {
                $page_number_type(self.0 / PAGE_SIZE)
            }

            pub fn next_page_number(&self) -> $page_number_type {
                $page_number_type((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
            }

            pub fn page_offset(&self) -> usize {
                self.0 & (PAGE_SIZE - 1)
            }

            pub fn is_aligned(&self) -> bool {
                (self.0 & (PAGE_SIZE - 1)) == 0
            }
        }
    };
}

address_impl!(VirtAddress, VirtPageNum);
address_impl!(PhysAddress, PhysPageNum);

impl PhysPageNum {
    #[inline(always)]
    /// get the page base ptr ppn point to
    pub fn page_base_ptr(&self) -> *mut u8 {
        (self.0 << PAGE_SIZE_BITS) as *mut u8
    }

    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.page_base_ptr() as *mut PageTableEntry,
                PAGE_SIZE / core::mem::size_of::<PageTableEntry>(),
            )
        }
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.page_base_ptr(), PAGE_SIZE) }
    }
}

impl VirtPageNum {
    /// return indexes in page table
    ///
    /// returned indexes array determined by const PAGE_TRANSLATION_LEVELS
    ///
    /// for sv39 now it's 3
    pub fn indexes(&self) -> [usize; PAGE_TRANSLATION_LEVELS] {
        let mask = (1 << VPN_INDEX_WIDTH_BITS) - 1;
        let mut vpn = self.0;
        let mut ret = [0; PAGE_TRANSLATION_LEVELS];
        for i in (0..PAGE_TRANSLATION_LEVELS).rev() {
            ret[i] = vpn & mask;
            vpn >>= VPN_INDEX_WIDTH_BITS;
        }
        ret
    }

    /// return extended indexes in gstage pagetable
    ///
    /// gstage pte has 2 more bits in first translation level
    pub fn extended_indexes(&self) -> [usize; PAGE_TRANSLATION_LEVELS] {
        let mut ret = self.indexes();

        ret[0] = (self.0 >> (VPN_INDEX_WIDTH_BITS * (PAGE_TRANSLATION_LEVELS - 1)))
            & ((1 << (VPN_INDEX_WIDTH_BITS + 2)) - 1);
        ret
    }

    #[inline]
    pub fn page_base_va(&self) -> VirtAddress {
        (self.0 << PAGE_SIZE_BITS).into()
    }
}

impl From<VirtPageNum> for VirtAddress {
    fn from(value: VirtPageNum) -> Self {
        value.page_base_va()
    }
}

impl From<VirtAddress> for VirtPageNum {
    fn from(value: VirtAddress) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.current_page_number()
    }
}

impl From<PhysPageNum> for PhysAddress {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}

impl From<PhysAddress> for PhysPageNum {
    fn from(value: PhysAddress) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.current_page_number()
    }
}

pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for PhysPageNum {
    #[inline(always)]
    fn step(&mut self) {
        self.0 += 1;
    }
}

impl StepByOne for VirtPageNum {
    #[inline(always)]
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Clone)]
pub struct PageRange<T>
where
    T: StepByOne + Clone + PartialOrd + PartialEq + Debug,
{
    start: T,
    end: T,
}

impl<T> PageRange<T>
where
    T: StepByOne + Clone + PartialOrd + PartialEq + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub fn get_start(&self) -> T {
        self.start.clone()
    }

    #[inline(always)]
    pub fn get_end(&self) -> T {
        self.end.clone()
    }

    pub fn iter(&self) -> PageRangeIterator<T> {
        PageRangeIterator {
            current: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

impl<T> IntoIterator for PageRange<T>
where
    T: StepByOne + Clone + PartialOrd + PartialEq + Debug,
{
    type Item = T;
    type IntoIter = PageRangeIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        PageRangeIterator {
            current: self.start,
            end: self.end,
        }
    }
}

/// iterator used for iter in both PPN & VPN
pub struct PageRangeIterator<T>
where
    T: StepByOne + Clone + PartialOrd + PartialEq + Debug,
{
    current: T,
    end: T,
}

impl<T> Iterator for PageRangeIterator<T>
where
    T: StepByOne + Clone + PartialOrd + PartialEq + Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current != self.end {
            let ret = self.current.clone();
            self.current.step();
            Some(ret)
        } else {
            None
        }
    }
}

pub type VPNRange = PageRange<VirtPageNum>;
pub type PPNRange = PageRange<PhysPageNum>;
