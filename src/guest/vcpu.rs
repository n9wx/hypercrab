use crate::arch::TrapContext;
#[derive(Clone, Copy)]
pub struct VCpu {
    context: TrapContext,
}

impl VCpu {
    pub fn new(context: TrapContext) -> Self {
        Self { context }
    }

    #[inline(always)]
    pub fn get_ctx_ptr(&mut self) -> *mut TrapContext {
        &mut self.context
    }
}
