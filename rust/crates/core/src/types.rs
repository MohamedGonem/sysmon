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
    pub available_mb: u64,
    pub buffers_mb: u64,
    pub cached_mb: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStats {
    pub memory: MemStat,
    pub swap: Option<MemStat>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
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
    pub load_average: LoadAverage,
    pub context_switches: u64,
    pub processes_running: u64,
    pub usage_percentage: f64,
    pub core_stats: Vec<CoreStat>,
    pub usage: UsageStat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreStat {
    pub id: u64,
    pub usage_percentage: f64,
    pub temperature: Option<f64>,
    pub current_frequency: Option<f64>,
    pub governor: Option<String>,
    pub online: bool,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BatteryUnit {
    #[default]
    Energy,
    Charge,
}

impl BatteryUnit {
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Energy => "uWh",
            Self::Charge => "uAh",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatteryStat {
    pub percentage: u64,
    pub status: String,
    pub energy_now: Option<u64>,
    pub energy_full: Option<u64>,
    pub energy_full_design: Option<u64>,
    pub power_now: Option<u64>,
    pub voltage_now: Option<u64>,
    pub cycle_count: Option<u64>,
    pub health_percentage: Option<f64>,
    pub energy_unit: BatteryUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkStats {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub errors_in: u64,
    pub errors_out: u64,
    pub drops_in: u64,
    pub drops_out: u64,
    pub is_up: bool,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskStats {
    pub mounts: Vec<DiskMount>,
    pub devices: Vec<DiskDevice>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskMount {
    pub device: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub usage_percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskDevice {
    pub name: String,
    pub reads_completed: u64,
    pub writes_completed: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_speed: f64,
    pub write_speed: f64,
    pub is_rotational: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessStats {
    pub processes: Vec<ProcessStat>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessStat {
    pub pid: u32,
    pub name: String,
    pub state: char,
    pub cpu_usage_percentage: f64,
    pub memory_rss_bytes: u64,
    pub memory_percentage: f64,
    pub virtual_memory_bytes: u64,
    pub user: String,
    pub threads: u64,
    pub open_fds: Option<u64>,
    pub command_line: String,
    pub parent_pid: u32,
    pub nice: i32,
    pub start_time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorKind {
    Cpu,
    Disk,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemperatureSensor {
    pub chip: String,
    pub label: String,
    pub celsius: f64,
    pub kind: SensorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FanSensor {
    pub chip: String,
    pub label: String,
    pub rpm: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SensorStats {
    pub temperatures: Vec<TemperatureSensor>,
    pub fans: Vec<FanSensor>,
}
