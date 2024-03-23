//! constants , structures and functions for Sv39 page based virtual address space

use crate::constants::PAGE_SIZE_BITS;

pub const SV39_PA_WIDTH_BITS: usize = 56;
pub const SV39_PPN_WIDTH_BITS: usize = SV39_PA_WIDTH_BITS - PAGE_SIZE_BITS;
pub const SV39_VA_WIDTH_BITS: usize = 39;
pub const SV39_VPN_WIDTH_BITS: usize = SV39_VA_WIDTH_BITS - PAGE_SIZE_BITS;
pub const PAGE_TRANSLATION_LEVELS: usize = 3;
