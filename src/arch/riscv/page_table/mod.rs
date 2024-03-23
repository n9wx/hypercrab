use crate::constants::PAGE_SIZE;

mod address;
mod pte;
mod sv39;

pub const ROOT_PAGE_TABLE_SIZE: usize = PAGE_SIZE * 4;
pub const VPN_INDEX_WIDTH_BITS: usize = 9;
