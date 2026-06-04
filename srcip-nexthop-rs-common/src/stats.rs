use core::fmt;

#[cfg(feature = "user")]
use aya::Pod;

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum StatType {
    Modified,
    Passed,
    Bad,
    StatsMax,
}

impl fmt::Display for StatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            StatType::Modified => "Modified",
            StatType::Passed => "Passed",
            StatType::Bad => "Bad",
            StatType::StatsMax => "StatsMax",
        };
        write!(f, "{}", name)
    }
}

impl From<u32> for StatType {
    fn from(value: u32) -> Self {
        match value {
            0 => StatType::Modified,
            1 => StatType::Passed,
            2 => StatType::Bad,
            _ => StatType::StatsMax,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Stat {
    pub pkt: u64,
    pub byt: u64,
}

#[cfg(feature = "user")]
unsafe impl Pod for Stat {}
