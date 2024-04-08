use riscv::register::{
    hstatus::{self, Hstatus},
    sstatus::{self, Sstatus, SPP},
};

#[repr(C)]
pub struct Context {
    // general purpose regs
    pub regs: [usize; 32],
    // vsstatus for the hart
    pub vsstatus: Sstatus,
    // hstatus for vcpu context
    pub hstatus: Hstatus,
    // trap pc in guest address space
    pub vsepc: usize,
    pub hgatp: usize,
    // stack ptr in vmm address space
    pub vmm_stack: usize,
    // address of trap_handler
    pub trap_handler: usize,
}

impl Context {
    #[inline(always)]
    pub fn set_sp(&mut self, sp: usize) {
        self.regs[2] = sp;
    }
    pub fn init_context(entry: usize, stack_ptr: usize, hgatp: usize, trap_handler: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::Supervisor);
        let mut hstatus = hstatus::read();
        unsafe {
            // return to vs mode
            hstatus.set_spv(true);
        }
        Self {
            regs: [0; 32],
            vsstatus: sstatus,
            hstatus,
            vsepc: entry,
            hgatp,
            vmm_stack: stack_ptr,
            trap_handler,
        }
    }
}
