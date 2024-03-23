mod page_table;

pub fn is_cpu_support() -> bool {
    use crate::constants::HYPERVISOR_EXTENSION;
    use crate::sbi::sbi_probe_extension;
    use crate::sbi::SBI_SUCCESS;

    sbi_probe_extension(HYPERVISOR_EXTENSION) == SBI_SUCCESS
}
