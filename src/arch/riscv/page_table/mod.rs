use crate::constants::PAGE_SIZE;
use riscv::register::satp;
use spin::Once;

pub mod address;
pub mod pte;
pub mod sv39;

pub use address::*;
pub use pte::*;
pub use sv39::*;

pub const ROOT_PAGE_TABLE_SIZE: usize = PAGE_SIZE * 4;
pub const VPN_INDEX_WIDTH_BITS: usize = 9;
pub const SECOND_STAGE_PAGE_TABLE_PAGE_NUMS: usize = 4;

pub(crate) static mut PAGE_MODE: Once<satp::Mode> = Once::new();

pub fn page_mode_probe() {
    let satp = satp::read();
    unsafe {
        PAGE_MODE.call_once(|| satp.mode());
    }
}
