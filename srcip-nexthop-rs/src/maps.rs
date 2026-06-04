use std::sync::{Arc, Mutex};

use aya::maps::{MapData, PerCpuArray, PerCpuHashMap};
use srcip_nexthop_rs_common::{hop::HopVal, stats::Stat};

pub struct Maps {
    pub stats: Arc<Mutex<PerCpuArray<MapData, Stat>>>,
    pub hops: Arc<Mutex<PerCpuHashMap<MapData, u32, HopVal>>>,
}
