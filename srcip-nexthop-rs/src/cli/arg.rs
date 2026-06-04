use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'i', long = "iface", default_value = None, help = "Overrides the network interface specified in the configuration file if set. Defaults to 'eth0' if not specified in the config file.")]
    pub iface: Option<String>,

    #[arg(
        short = 'c',
        long = "cfg",
        default_value = "/etc/srcip-nexthop/cfg.json",
        help = "The configuration file to use. Defaults to '/etc/srcip-nexthop/cfg.json'."
    )]
    pub cfg_str: String,

    #[arg(short = 'd', long = "duration", default_value = None, help = "The duration for which the program will run. If not specified, the program will run indefinitely.")]
    pub duration: Option<i32>,

    #[arg(
        short = 'l',
        long = "list",
        default_value_t = false,
        help = "List the configured hops and exit."
    )]
    pub list: bool,

    #[arg(
        short = 'n',
        long = "no-stats",
        default_value_t = false,
        help = "Disable statistics collection."
    )]
    pub no_stats: bool,
}
