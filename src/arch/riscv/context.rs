use riscv::register::hgatp::Hgatp;
use riscv::register::{
    hstatus::{self, Hstatus},
    sstatus::{self, Sstatus, SPP},
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrapContext {
    // general purpose regs
    pub regs: [usize; 32],
    // vsstatus for the hart
    pub sstatus: Sstatus,
    // trap pc in guest address space
    pub sepc: usize,
    // hstatus for vcpu context
    pub hstatus: Hstatus,
    // hgatp for vcpu context
    pub hgatp: Hgatp,
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
            sstatus,
            hstatus,
            sepc: entry,
            guest_hyp_stack: stack_ptr,
            trap_handler,
            hgatp: Hgatp::from_bits(hgatp),
        }
    }
}
