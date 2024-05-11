use crate::arch::TrapContext;
#[derive(Clone, Copy)]
pub struct VCpu {
    context: TrapContext,
}

impl VCpu {
    pub fn new(context: TrapContext) -> Self {
        Self { context }
    }
}
