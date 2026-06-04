#[cfg(feature = "user")]
use aya::Pod;

use crate::ETH_ALEN;

pub const MAX_HOPS: usize = 1024;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct HopVal {
    pub next_hop: [u8; ETH_ALEN],
}

#[cfg(feature = "user")]
unsafe impl Pod for HopVal {}
