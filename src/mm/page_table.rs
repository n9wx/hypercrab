use crate::mm::{PTEFlags, PageTableEntry, PageTableWalkIter, PhysPageNum, VirtPageNum};

pub trait PageTable: Clone {
    fn new() -> Self;

    /// use pre allocated root page tale if in need
    fn new_from_kernel() -> Self;

    /// map virt page to phys page
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, pte_flags: PTEFlags);

    /// unmap virt page
    fn unmap(&mut self, vpn: VirtPageNum);

    fn translate(&mut self, vpn: VirtPageNum) -> Option<PageTableEntry>;

    ///walk through page table in specify vpn range
    fn page_table_walk(&self, start_vpn: VirtPageNum, end_vpn: VirtPageNum) -> PageTableWalkIter;

    /// page table root token
    ///
    /// usually page table register
    fn token(&self) -> usize;
}

pub trait GStagePageTable: PageTable {
    /// create second page table for guest
    fn new_guest_stage() -> Self;
}
