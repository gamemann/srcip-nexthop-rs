pub mod cli;
pub mod config;
pub mod maps;

use std::{
    io,
    net::Ipv4Addr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Result, anyhow};
use aya::{
    maps::{MapData, PerCpuArray, PerCpuHashMap, PerCpuValues},
    programs::{SchedClassifier, TcAttachType, tc},
};
use clap::Parser;
#[rustfmt::skip]
use log::{debug, warn};
use srcip_nexthop_rs_common::{
    ETH_ALEN,
    hop::HopVal,
    stats::{
        Stat,
        StatType::{self, StatsMax},
    },
};
use tokio::{
    fs::write,
    io::Interest,
    select, signal, task,
    time::{Instant, interval, sleep},
};

use crate::{cli::arg::Args, config::cfg::Config, maps::Maps};

use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse arguments first.
    let Args {
        iface,
        cfg_str,
        list,
        no_stats,
        duration,
    } = Args::parse();

    // Initialize config with default values first.
    let mut cfg = Config::default();

    let iface = iface.unwrap_or_else(|| cfg.iface.clone());

    // Attempt to load config from file.
    cfg.from_file(&cfg_str)
        .await
        .map_err(|e| anyhow!("Failed to load config from {}: {}", cfg_str, e))?;

    // List settings if list flag is set and exit.
    if list {
        cfg.list();

        return Ok(());
    }

    // Initialize logger first.
    env_logger::init();

    // Raise rlimit.
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };

    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };

    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {ret}");
    }

    // Load eBPF program.
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/srcip-nexthop-rs"
    )))?;

    // Initialize eBPF logger.
    match aya_log::EbpfLogger::init(&mut ebpf) {
        Err(e) => {
            // This can happen if you remove all log statements from your eBPF program.
            warn!("failed to initialize eBPF logger: {e}");
        }
        Ok(logger) => {
            let mut logger = tokio::io::unix::AsyncFd::with_interest(logger, Interest::READABLE)?;
            task::spawn(async move {
                loop {
                    let mut guard = logger.readable_mut().await.unwrap();
                    guard.get_inner_mut().flush();
                    guard.clear_ready();
                }
            });
        }
    }

    // Create clsact qdisc if not exists.
    let _ = tc::qdisc_add_clsact(&iface);

    let prog: &mut SchedClassifier = ebpf
        .program_mut("srcip_nexthop_rs")
        .unwrap()
        .try_into()
        .map_err(|e| anyhow!("Failed to convert program to SchedClassifier: {}", e))?;

    // Load the program into the kernel.
    prog.load()
        .map_err(|e| anyhow!("Failed to load program: {}", e))?;

    // Attach to TC egress hook.
    prog.attach(&iface, TcAttachType::Egress)
        .map_err(|e| anyhow!("Failed to attach program to TC egress hook: {}", e))?;

    // Retrieve maps and store them in context.
    let map_hops = ebpf
        .take_map("MAP_HOP")
        .ok_or_else(|| anyhow!("Failed to find map MAP_HOP map..."))?;

    let map_hops = PerCpuHashMap::<MapData, u32, HopVal>::try_from(map_hops)
        .map_err(|e| anyhow!("Failed to convert MAP_HOP to PerCpuHashMap: {}", e))?;

    let map_stats = ebpf
        .take_map("MAP_STATS")
        .ok_or_else(|| anyhow!("Failed to find map MAP_STATS map..."))?;

    let map_stats = PerCpuArray::<MapData, Stat>::try_from(map_stats)
        .map_err(|e| anyhow!("Failed to convert MAP_STATS to PerCpuArray: {}", e))?;

    // Store these maps inside of maps context.
    let maps_ctx = Arc::new(Maps {
        stats: Arc::new(Mutex::new(map_stats)),
        hops: Arc::new(Mutex::new(map_hops)),
    });

    // Now store the next hop values from the config map.
    for hop in cfg.hops.iter() {
        // We need to convert our IP string to a u32 in network byte order to use as the key for the map.
        let addr: Ipv4Addr = hop
            .src_ip
            .parse()
            .map_err(|e| anyhow!("Failed to parse IP address {}: {}", hop.src_ip, e))?;

        let key = u32::from_be_bytes(addr.octets());

        // We need to split the hop string by colon for MAC address.
        let parts: Vec<&str> = hop.next_hop.split(":").collect();

        if parts.len() != 6 {
            return Err(anyhow!(
                "Invalid MAC address format for next hop {}: expected 6 octets separated by colons",
                hop.next_hop
            ));
        }

        // We need to convert our MAC address string to a [u8; 6] array to store in the map.
        let new_hop_eth: [u8; ETH_ALEN] = parts
            .iter()
            .map(|part| {
                u8::from_str_radix(part, 16)
                    .map_err(|e| anyhow!("Failed to parse MAC address part {}: {}", part, e))
            })
            .collect::<Result<Vec<u8>>>()?
            .try_into()
            .map_err(|e| anyhow!("Failed to convert MAC address to [u8; 6]: {:?}", e))?;

        // Create base hop value.
        let val = HopVal {
            next_hop: new_hop_eth,
        };

        // We need to create a separate per CPU value for each CPU since this is a per CPU map.
        let val_pcpu = PerCpuValues::try_from(vec![val; num_cpus::get()])
            .map_err(|e| anyhow!("Failed to create PerCpuValues for hop value: {}", e))?;

        // Now insert this into the map.
        let mut map = maps_ctx
            .hops
            .lock()
            .map_err(|e| anyhow!("Failed to lock hops map mutex: {}", e))?;

        match map.insert(key, val_pcpu, 0) {
            Ok(_) => {
                println!(
                    "Inserted hop: {} => {} (key: {}, val: {:?})",
                    hop.src_ip, hop.next_hop, key, new_hop_eth
                );
            }
            Err(e) => {
                return Err(anyhow!("Failed to insert hop value into map: {}", e));
            }
        }
    }

    let running = Arc::new(AtomicBool::new(true));

    // Spawn task to show stats.
    {
        let maps_ctx = maps_ctx.clone();
        let running = running.clone();

        task::spawn(async move {
            // Get start time.
            let start = Instant::now();

            while running.load(Ordering::Relaxed) {
                // Sleep for 1 second.
                sleep(Duration::from_secs(1)).await;

                if let Some(duration) = duration {
                    if start.elapsed() >= Duration::from_secs(duration as u64) {
                        println!(
                            "Specified duration of {:?} has elapsed, killing program...",
                            duration
                        );

                        running.store(false, Ordering::Relaxed);

                        break;
                    }
                }

                // We need to process contents inside of its own block so we don't perform async file write while holding the std mutex lock on the map (not allowed).
                let contents = {
                    // Now read the stats from the map and print them out.
                    let map = maps_ctx
                        .stats
                        .lock()
                        .map_err(|e| anyhow!("Failed to lock stats map mutex: {}", e))
                        .unwrap();

                    // Create contents for if we need to store the contents.
                    let mut new_contents = String::new();

                    for i in 0..StatsMax as u32 {
                        let values = match map.get(&i, 0) {
                            Ok(values) => values,
                            Err(e) => {
                                warn!("Failed to get stats for {:?}: {}", i, e);
                                continue;
                            }
                        };

                        // Sum up the per CPU values for this stat type.
                        let tot_pkt: u64 = values.iter().map(|stat| stat.pkt).sum();
                        let tot_byt: u64 = values.iter().map(|stat| stat.byt).sum();

                        let stat = StatType::from(i);

                        new_contents.push_str(&format!(
                            "{}Pkt:{}\n{}Byt:{}\n",
                            stat, tot_pkt, stat, tot_byt
                        ));
                    }

                    new_contents
                };

                if !no_stats {
                    // Clear screen and move cursor to top
                    print!("\x1B[2J\x1B[1;1H");

                    // Print the contents to stdout.
                    print!("Stats:\n{}", contents);

                    // Finally flush.
                    io::stdout()
                        .flush()
                        .map_err(|e| anyhow!("Failed to flush stdout: {}", e))
                        .unwrap();
                }

                // If we have a counters store, write the contents to the file.
                if let Some(counters_store) = &cfg.counters_store {
                    if let Err(e) = write(counters_store, &contents).await {
                        warn!("Failed to write counters to {}: {}", counters_store, e);
                    }
                }
            }
        });
    }

    let mut tick = interval(Duration::from_secs(1));

    loop {
        select! {
            _ = tick.tick() => {
                // Check if the program is still running every tick and exit if not.
                if !running.load(Ordering::Relaxed) {
                    break;
                }
            }
            _ = signal::ctrl_c() => {
                println!("Received Ctrl-C, exiting...");

                running.store(false, Ordering::Relaxed);

                break;
            }
        }
    }

    // Exit gracefully.
    // We don't need to cleanup the eBPF program or maps since Aya handles that automatically.
    println!("Exiting...");

    Ok(())
}
