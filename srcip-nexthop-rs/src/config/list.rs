use crate::config::cfg::Config;

impl Config {
    pub fn list(&self) {
        println!("Listing settings...");

        println!(
            "Counters store: {}",
            self.counters_store.as_deref().unwrap_or("None")
        );

        println!("Hops:");
        for hop in &self.hops {
            println!("\t{} => {}", hop.src_ip, hop.next_hop);
        }
    }
}
