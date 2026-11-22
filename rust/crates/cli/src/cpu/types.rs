pub struct CPUStat {
    pub vendor_id: String,
    pub model_name: String,
    pub cache_levels: Vec<[String; 3]>,
    pub governor: String,
    pub flags: Option<Vec<String>>,
    pub physical_cores_count: u64,
    pub logical_cores_count: u64,
    pub current_frequency: f64,
    pub min_frequency: f64,
    pub max_frequency: f64,
    pub temperature: f64,
    pub usage_percentage: f64,
    pub core_stats: Vec<CoreStat>,
    pub usage: UsageStat,
}

pub struct CoreStat {
    pub id: u64,
    pub usage_percentage: f64,
    pub temperature: Option<f64>,
    pub usage: UsageStat,
}

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
