#![no_std]
#![no_main]

mod stats;

use aya_ebpf::{
    bindings::{TC_ACT_PIPE, TC_ACT_SHOT},
    macros::{classifier, map},
    maps::{PerCpuArray, PerCpuHashMap},
    programs::TcContext,
};
use srcip_nexthop_rs_common::{
    hop::{HopVal, MAX_HOPS},
    stats::{
        Stat,
        StatType::{self, StatsMax},
    },
};

use network_types::{
    eth::{EthHdr, EtherType},
    ip::Ipv4Hdr,
};

use crate::stats::inc_stats;

#[map]
pub static MAP_STATS: PerCpuArray<Stat> = PerCpuArray::with_max_entries(StatsMax as u32, 0);

#[map]
pub static MAP_HOP: PerCpuHashMap<u32, HopVal> =
    PerCpuHashMap::with_max_entries(MAX_HOPS as u32, 0);

#[classifier]
pub fn srcip_nexthop_rs(ctx: TcContext) -> i32 {
    let pkt_len = ctx.data_end() - ctx.data();

    // First, load and check the ethernet header.
    let eth: EthHdr = match ctx.load(0) {
        Ok(eth_hdr) => eth_hdr,
        Err(_) => {
            inc_stats(StatType::Bad, pkt_len as u64);

            return TC_ACT_SHOT;
        }
    };

    if ctx.data() + EthHdr::LEN > ctx.data_end() {
        inc_stats(StatType::Bad, pkt_len as u64);

        return TC_ACT_SHOT;
    }

    if eth.ether_type != EtherType::Ipv4 as u16 {
        inc_stats(StatType::Passed, pkt_len as u64);

        return TC_ACT_PIPE;
    }

    // Now load and check the IP header.
    let iph: Ipv4Hdr = match ctx.load(14) {
        Ok(ip_hdr) => ip_hdr,
        Err(_) => {
            inc_stats(StatType::Bad, pkt_len as u64);

            return TC_ACT_SHOT;
        }
    };

    if ctx.data() + EthHdr::LEN + Ipv4Hdr::LEN > ctx.data_end() {
        inc_stats(StatType::Bad, pkt_len as u64);

        return TC_ACT_SHOT;
    }

    // Do lookup on the hop map using the source IP address as the key.
    let key = u32::from_be_bytes(iph.src_addr);

    let next_hop = match MAP_HOP.get_ptr_mut(key) {
        Some(hop_val) => unsafe { &mut *hop_val },
        None => {
            inc_stats(StatType::Passed, pkt_len as u64);

            return TC_ACT_PIPE;
        }
    };

    if ctx.data() + EthHdr::LEN > ctx.data_end() {
        inc_stats(StatType::Bad, pkt_len as u64);

        return TC_ACT_SHOT;
    }

    // Replace the next hop MAC address in the Ethernet header with the value from the map.
    let eth_ptr = ctx.data() as *mut EthHdr;

    unsafe {
        (*eth_ptr).dst_addr = next_hop.next_hop;
    }

    inc_stats(StatType::Modified, pkt_len as u64);

    TC_ACT_PIPE
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
