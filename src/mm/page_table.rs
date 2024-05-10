use crate::arch::page_table::{
    PPNRange, PTEFlags, PageTableEntry, PageTableWalkIter, PhysPageNum, StepByOne, VPNRange,
    VirtPageNum,
};
use core::ptr::NonNull;

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

    /// just walk page table add find specify pte,return mut reference
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry>;

    /// walk page table ,find specify pte,if search path is illegal,create it and modify pagetable
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry>;
}

pub trait GStagePageTable: PageTable {
    /// create second page table for guest
    fn new_guest_stage() -> Self;
}

/// iterator combined guest pagetable and host pagetable
///
/// walk through pagetables and fill g stage pagetable for guest
pub struct CombinedWalker<'a, P: PageTable, G: GStagePageTable + 'a> {
    host_page_table: &'a P,
    guest_page_table: NonNull<G>,
    current_hvpn: VirtPageNum,
    current_gppn: PhysPageNum,
    end_hvpn: VirtPageNum,
}

impl<'a, P: PageTable, G: GStagePageTable + 'a> CombinedWalker<'a, P, G> {
    pub fn new(
        host_page_table: &'a P,
        guest_page_table: &'a mut G,
        hvpn_range: VPNRange,
        gppn_range: PPNRange,
    ) -> Self {
        assert_eq!(
            hvpn_range.get_end().0 - hvpn_range.get_start().0,
            gppn_range.get_end().0 - gppn_range.get_start().0,
            "mem region size in host address space and guest address space not equal"
        );

        Self {
            host_page_table,
            guest_page_table: unsafe { NonNull::new_unchecked(guest_page_table) },
            current_hvpn: hvpn_range.get_start(),
            current_gppn: gppn_range.get_start(),
            end_hvpn: hvpn_range.get_end(),
        }
    }
}

impl<'a, P: PageTable, G: GStagePageTable + 'a> Iterator for CombinedWalker<'a, P, G> {
    // return (gppn , hvpn)
    type Item = (&'a mut PageTableEntry, &'a PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_hvpn == self.end_hvpn {
            None
        } else {
            unsafe {
                let Some(guest_pte): Option<&'a mut PageTableEntry> = self
                    .guest_page_table
                    .as_mut()
                    .find_pte_create(VirtPageNum(self.current_gppn.0))
                else {
                    return None;
                };

                let Some(host_pte): Option<&'a mut PageTableEntry> =
                    self.host_page_table.find_pte(self.current_hvpn)
                else {
                    return None;
                };

                self.current_hvpn.step();
                self.current_gppn.step();
                Some((guest_pte, host_pte))
            }
        }
    }
}

pub fn fill_guest_page_table<P: PageTable, G: GStagePageTable>(walker: CombinedWalker<P, G>) {
    for (guest_pte, host_pte) in walker {
        let host_ppn = host_pte.ppn();
        // todo 暂时先给整个guest address space rwx权限,应该有方法限制吧
        *guest_pte = PageTableEntry::new(host_ppn, PTEFlags::R | PTEFlags::W | PTEFlags::X);
    }
}
