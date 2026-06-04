use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'i', long = "iface", default_value = None)]
    pub iface: Option<String>,

    #[arg(
        short = 'c',
        long = "cfg",
        default_value = "/etc/srcip-nexthop/cfg.json"
    )]
    pub cfg_str: String,

    #[arg(short = 'l', long = "list", default_value_t = false)]
    pub list: bool,

    #[arg(short = 'n', long = "no-stats", default_value_t = false)]
    pub no_stats: bool,
}
