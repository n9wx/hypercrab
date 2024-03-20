use core::arch::asm;

// sbi return value
pub const SBI_SUCCESS: usize = 0;
pub const SBI_ERR_FAILED: isize = -1;
pub const SBI_ERR_NOT_SUPPORTED: isize = -2;
pub const SBI_ERR_INVALID_PARAM: isize = -3;
pub const SBI_ERR_DENIED: isize = -4;
pub const SBI_ERR_INVALID_ADDRESS: isize = -5;
pub const SBI_ERR_ALREADY_AVAILABLE: isize = -6;
pub const SBI_ERR_ALREADY_STARTED: isize = -7;
pub const SBI_ERR_ALREADY_STOPPED: isize = -8;
pub const SBI_ERR_NO_SHMEM: isize = -9;

// base extension id & functions id
pub const SBI_BASE_EXTENSION: usize = 0x10;
pub const PROBE_CPU_EXTENSION: usize = 0x3;

pub const RUSTSBI_PUT_CHAR_EXTENSION: usize = 0x1;
pub const RUSTSBI_GET_CHAR_EXTENSION: usize = 0x2;

// ascii represent of "SRST"
pub const SBI_RESET_EXTENSION: usize = 0x53525354;
// sbi_system_reset(reset_type:u32,reset_reason:u32);
pub const SYSTEM_RESET: usize = 0x0;
// system reset types
pub const SHUTDOWN: usize = 0;
// system reset reason
pub const NO_REASON: usize = 0;


#[inline(always)]
pub fn sbi_call(sbi_extension: usize, function_id: usize, args: [usize; 3]) -> usize {
    let mut ret: usize;
    unsafe {
        asm!(
        "ecall",
        in("a7") sbi_extension,
        in("a6") function_id,
        inlateout("a0") args[0] => ret,
        in("a1") args[1],
        in("a2") args[2]
        );
    }
    ret
}

pub fn sbi_probe_extension(extension_id: usize) -> usize {
    sbi_call(SBI_BASE_EXTENSION, PROBE_CPU_EXTENSION, [extension_id, 0, 0])
}

pub fn sbi_shutdown() -> ! {
    sbi_call(SBI_RESET_EXTENSION, SYSTEM_RESET, [SHUTDOWN, NO_REASON, 0]);
    unreachable!()
}

// use non standard sbi call put char in console(qemu)
pub fn sbi_put_char(c: usize) { sbi_call(RUSTSBI_PUT_CHAR_EXTENSION, 0, [c, 0, 0]); }

// use non standard sbi call get char from console(qemu)
pub fn sbi_get_char() -> usize { sbi_call(RUSTSBI_GET_CHAR_EXTENSION, 0, [0, 0, 0]) }

