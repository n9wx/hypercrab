#![allow(dead_code)]

use raw_cpuid::CpuId;

pub fn is_cpu_support() -> bool {
    if let Some(feature) = CpuId::new().get_feature_info() {
        feature.has_vmx()
    } else {
        false
    }
}

pub struct VmxPerCpuState {
    vmx_region: (),
    revision: u32,
}

impl VmxPerCpuState {
    pub fn enable_hardware(&self) {}
}