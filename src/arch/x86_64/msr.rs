use x86::msr::{rdmsr, wrmsr};

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum Msr {
    IA32_FEATURE_CONTROL = 0x3a,
    IA32_VMX_BASE = 0x480,

    IA32_VMX_CR0_FIXED0 = 0x486,
    IA32_VMX_CR0_FIXED1 = 0x487,

    IA32_VMX_CR4_FIXED0 = 0x488,
    IA32_VMX_CR4_FIXED1 = 0x489,
}

impl Msr {
    #[inline(always)]
    pub fn read(self) -> u64 {
        unsafe { rdmsr(self as _) }
    }

    #[inline(always)]
    pub fn write(self, val: u64) {
        unsafe { wrmsr(self as _, val) }
    }
}