use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Hop {
    pub src_ip: String,
    pub next_hop: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub counters_store: Option<String>,
    pub hops: Vec<Hop>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            counters_store: Some("./counters.txt".to_string()),
            hops: Vec::new(),
        }
    }
}
