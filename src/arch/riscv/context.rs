use riscv::register::hgatp::Hgatp;
use riscv::register::{
    hstatus::{self, Hstatus},
    sstatus::{self, Sstatus, SPP},
};

// register type used in riscv::register is not ffi safe,so we just use usize
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrapContext {
    // general purpose regs
    pub regs: [usize; 32],
    // sstatus for the hart
    pub sstatus: usize,
    // trap pc in guest address space
    pub sepc: usize,
    // hstatus for vcpu context
    pub hstatus: usize,
    // hgatp for vcpu context
    pub hgatp: usize,
    // stack ptr in hypervisor address space
    pub guest_hyp_stack: usize,
    // address of trap_handler
    pub trap_handler: usize,
}

impl TrapContext {
    #[inline(always)]
    pub fn set_sp(&mut self, sp: usize) {
        self.regs[2] = sp;
    }

    /// set init context, include stack in hyp address space hgatp for the vcpu,return pl and address
    /// in guest address space
    pub fn init_context(entry: usize, stack_ptr: usize, hgatp: usize, trap_handler: usize) -> Self {
        let mut sstatus = sstatus::read();
        // return to s mode
        sstatus.set_spp(SPP::Supervisor);
        let mut hstatus = hstatus::read();
        // return to virtual pl(VS or VU)
        hstatus.set_spv(true);
        Self {
            regs: [0; 32],
            sstatus: sstatus.bits(),
            hstatus: hstatus.bits(),
            sepc: entry,
            guest_hyp_stack: stack_ptr,
            trap_handler,
            hgatp,
        }
    }
}
