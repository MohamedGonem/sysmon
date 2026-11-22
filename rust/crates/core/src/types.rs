#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysStat {
    pub hostname: String,
    pub kernel: String,
    pub distro: String,
    pub arch: String,
    pub uptime_secs: u64,
    pub boot_time: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemStat {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStats {
    pub memory: MemStat,
    pub swap: Option<MemStat>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CPUStat {
    pub vendor_id: String,
    pub model_name: String,
    pub cache_levels: Vec<[String; 3]>,
    pub governor: String,
    pub flags: Vec<String>,
    pub physical_cores_count: u64,
    pub logical_cores_count: u64,
    pub current_frequency: f64,
    pub min_frequency: f64,
    pub max_frequency: f64,
    pub temperature: Option<f64>,
    pub usage_percentage: f64,
    pub core_stats: Vec<CoreStat>,
    pub usage: UsageStat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreStat {
    pub id: u64,
    pub usage_percentage: f64,
    pub temperature: Option<f64>,
    pub usage: UsageStat,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UsageStat {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryStat {
    pub percentage: u64,
    pub status: String,
}
