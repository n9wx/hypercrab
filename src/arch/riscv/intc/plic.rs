const MAX_INT_SOURCE_ID: usize = 1023;
const MAX_CONTEXT: usize = 15872;
const PENDING_BASE: usize = 0x1000;
const ENABLE_BASE: usize = 0x2000;

pub struct Plic {
    base_addr: usize,
}

impl Plic {
    #[inline(always)]
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    pub unsafe fn priority_ptr(&self, int_source_id: usize) -> *mut u32 {
        assert!(int_source_id > 0 && int_source_id <= MAX_INT_SOURCE_ID);
        (self.base_addr + int_source_id * 4) as *mut u32
    }
}
