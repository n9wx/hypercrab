#[cfg(target_arch = "riscv64")]
pub use crate::arch::page_table::*;

pub trait PageTable: Clone {
    fn new() -> Self;

    /// use pre allocated root page tale if in need
    fn new_from_kernel() -> Self;

    /// map virt page to phys page
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, pte_flags: PTEFlags);

    /// unmap virt page
    fn unmap(&mut self, vpn: VirtPageNum);

    fn translate(&mut self, vpn: VirtPageNum) -> Option<PageTableEntry>;

    /// page table root token
    /// usually page table register
    fn token(&self) -> usize;
}
