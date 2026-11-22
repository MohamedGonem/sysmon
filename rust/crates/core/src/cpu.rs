use std::time::Duration;

use crate::types::{CPUStat, CoreStat, UsageStat};

const HWMON_COUNT: u64 = 20;
const CACHE_INDEXES: u64 = 4;
const INTERESTING_FLAGS: [&str; 11] = [
    "avx",
    "avx2",
    "avx512f",
    "sse4_1",
    "fma",
    "aes",
    "sgx",
    "vmx",
    "svm",
    "hypervisor",
    "ht",
];

impl UsageStat {
    /// Parse a `cpu` or `cpuN` line from `/proc/stat`.
    pub fn from_proc_stat_line(line: &str) -> Self {
        let mut numbers = line
            .split_whitespace()
            .skip(1)
            .map(|number| number.parse::<u64>().unwrap_or(0));

        Self {
            user: numbers.next().unwrap_or(0),
            nice: numbers.next().unwrap_or(0),
            system: numbers.next().unwrap_or(0),
            idle: numbers.next().unwrap_or(0),
            iowait: numbers.next().unwrap_or(0),
            irq: numbers.next().unwrap_or(0),
            softirq: numbers.next().unwrap_or(0),
            steal: numbers.next().unwrap_or(0),
            guest: numbers.next().unwrap_or(0),
            guest_nice: numbers.next().unwrap_or(0),
        }
    }

    /// Total number of clock ticks accounted for by this sample.
    pub fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
            + self.guest
            + self.guest_nice
    }

    /// Ticks spent idle, including time waiting for I/O.
    pub fn idle_total(&self) -> u64 {
        self.idle + self.iowait
    }

    /// CPU usage percentage between an earlier sample and this sample.
    pub fn usage_since(&self, earlier: &Self) -> f64 {
        let total = self.total().saturating_sub(earlier.total());
        if total == 0 {
            return 0.0;
        }

        let idle = self.idle_total().saturating_sub(earlier.idle_total());
        100.0 * (1.0 - idle as f64 / total as f64)
    }
}

/// A single read of `/proc/stat`, containing the aggregate CPU line and every core line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuTimes {
    pub total: UsageStat,
    pub cores: Vec<UsageStat>,
}

impl CpuTimes {
    /// Read and parse `/proc/stat`.
    pub fn read() -> Option<Self> {
        let content = std::fs::read_to_string("/proc/stat").ok()?;
        Self::parse(&content)
    }

    /// Parse the contents of `/proc/stat`.
    pub fn parse(content: &str) -> Option<Self> {
        let total = content
            .lines()
            .find(|line| line.split_whitespace().next() == Some("cpu"))
            .map(UsageStat::from_proc_stat_line)?;

        let cores = content
            .lines()
            .filter(|line| is_core_line(line))
            .map(UsageStat::from_proc_stat_line)
            .collect();

        Some(Self { total, cores })
    }

    /// Aggregate CPU usage percentage between an earlier sample and this sample.
    pub fn usage_since(&self, earlier: &Self) -> f64 {
        self.total.usage_since(&earlier.total)
    }

    /// Per-core CPU usage percentages between an earlier sample and this sample.
    pub fn core_usages_since(&self, earlier: &Self) -> Vec<f64> {
        self.cores
            .iter()
            .zip(&earlier.cores)
            .map(|(later, earlier)| later.usage_since(earlier))
            .collect()
    }

    /// Sample aggregate CPU usage over a window.
    pub fn sample_usage(window: Duration) -> Option<f64> {
        let (earlier, later) = Self::sample_pair(window)?;
        Some(later.usage_since(&earlier))
    }

    /// Sample every core's usage over a window.
    pub fn sample_core_usages(window: Duration) -> Option<Vec<f64>> {
        let (earlier, later) = Self::sample_pair(window)?;
        Some(later.core_usages_since(&earlier))
    }

    /// Sample aggregate and per-core usage over one shared window.
    pub fn sample_usage_and_cores(window: Duration) -> Option<(f64, Vec<f64>)> {
        let (earlier, later) = Self::sample_pair(window)?;
        Some((
            later.usage_since(&earlier),
            later.core_usages_since(&earlier),
        ))
    }

    fn sample_pair(window: Duration) -> Option<(Self, Self)> {
        let earlier = Self::read()?;
        std::thread::sleep(window);
        let later = Self::read()?;
        Some((earlier, later))
    }

    fn core_stats_since(&self, earlier: &Self) -> Vec<CoreStat> {
        self.cores
            .iter()
            .zip(&earlier.cores)
            .enumerate()
            .map(|(id, (later, earlier))| CoreStat::from_times(id as u64, earlier, later))
            .collect()
    }
}

impl CPUStat {
    /// Sample all CPU information over a window.
    ///
    /// Core details are only collected when `include_cores` is true.
    pub fn sample(window: Duration, include_cores: bool) -> Option<Self> {
        let (earlier, later) = CpuTimes::sample_pair(window)?;
        let info = CpuInfo::read();

        Some(Self {
            vendor_id: info.vendor_id,
            model_name: info.model_name,
            cache_levels: (0..CACHE_INDEXES).map(cache_level).collect(),
            governor: governor(0),
            flags: info.flags,
            physical_cores_count: info.physical_cores,
            logical_cores_count: info.logical_cores,
            current_frequency: read_frequency(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq",
            ),
            min_frequency: read_frequency("/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq"),
            max_frequency: read_frequency("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq"),
            temperature: cpu_temperature(),
            usage_percentage: later.usage_since(&earlier),
            core_stats: if include_cores {
                later.core_stats_since(&earlier)
            } else {
                Vec::new()
            },
            usage: later.total,
        })
    }
}

impl CoreStat {
    /// Sample every core over a window.
    pub fn sample(window: Duration) -> Option<Vec<Self>> {
        let (earlier, later) = CpuTimes::sample_pair(window)?;
        Some(later.core_stats_since(&earlier))
    }

    fn from_times(id: u64, earlier: &UsageStat, later: &UsageStat) -> Self {
        Self {
            id,
            usage_percentage: later.usage_since(earlier),
            temperature: core_temperature(id),
            usage: *later,
        }
    }
}

/// Static CPU information read once from `/proc/cpuinfo`.
struct CpuInfo {
    vendor_id: String,
    model_name: String,
    flags: Vec<String>,
    physical_cores: u64,
    logical_cores: u64,
}

impl CpuInfo {
    fn read() -> Self {
        let content = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        Self::parse(&content)
    }

    fn parse(content: &str) -> Self {
        let field = |key: &str| {
            content
                .lines()
                .find(|line| line.starts_with(key))
                .and_then(|line| line.split_once(':'))
                .map(|(_, value)| value.trim().to_string())
        };

        let vendor_id = field("vendor_id").unwrap_or_else(|| "Unknown".to_string());
        let model_name = field("model name").unwrap_or_else(|| "Unknown".to_string());
        let flags = field("flags")
            .map(|flags| {
                flags
                    .split_whitespace()
                    .filter(|flag| INTERESTING_FLAGS.contains(flag))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let physical_cores = field("cpu cores")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1);
        let logical_cores = content
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count() as u64;

        Self {
            vendor_id,
            model_name,
            flags,
            physical_cores,
            logical_cores,
        }
    }
}

fn is_core_line(line: &str) -> bool {
    line.strip_prefix("cpu")
        .and_then(|rest| rest.chars().next())
        .is_some_and(|first| first.is_ascii_digit())
}

fn read_trimmed(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|content| content.trim().to_string())
}

fn read_frequency(path: &str) -> f64 {
    read_trimmed(path)
        .and_then(|value| value.parse::<f64>().ok())
        .map(|khz| khz / 1000.0)
        .unwrap_or(0.0)
}

fn governor(cpu: u64) -> String {
    read_trimmed(&format!(
        "/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor"
    ))
    .unwrap_or_else(|| "Unknown".to_string())
}

fn cache_level(index: u64) -> [String; 3] {
    ["type", "level", "size"].map(|kind| {
        read_trimmed(&format!(
            "/sys/devices/system/cpu/cpu0/cache/index{index}/{kind}"
        ))
        .unwrap_or_default()
    })
}

fn read_hwmon_temperature(input: &str) -> Option<f64> {
    for index in 0..HWMON_COUNT {
        let Ok(name) = std::fs::read_to_string(format!("/sys/class/hwmon/hwmon{index}/name"))
        else {
            continue;
        };
        let name = name.trim();
        if name != "coretemp" && name != "k10temp" {
            continue;
        }

        let Ok(raw) = std::fs::read_to_string(format!("/sys/class/hwmon/hwmon{index}/{input}"))
        else {
            continue;
        };
        let Ok(millidegrees) = raw.trim().parse::<f64>() else {
            continue;
        };

        return Some(millidegrees / 1000.0);
    }

    None
}

fn cpu_temperature() -> Option<f64> {
    read_hwmon_temperature("temp1_input").or_else(|| {
        read_trimmed("/sys/class/thermal/thermal_zone0/temp")
            .and_then(|raw| raw.parse::<f64>().ok())
            .map(|millidegrees| millidegrees / 1000.0)
    })
}

fn core_temperature(core_id: u64) -> Option<f64> {
    read_hwmon_temperature(&format!("temp{}_input", core_id + 2))
}

#[cfg(test)]
mod tests {
    use super::{CpuInfo, is_core_line};

    #[test]
    fn parses_cpuinfo_fields() {
        let content = "\
processor\t: 0
vendor_id\t: GenuineIntel
model name\t: Test CPU
cpu cores\t: 4
flags\t\t: fpu vme avx avx2 ht
processor\t: 1
";
        let info = CpuInfo::parse(content);

        assert_eq!(info.vendor_id, "GenuineIntel");
        assert_eq!(info.model_name, "Test CPU");
        assert_eq!(info.physical_cores, 4);
        assert_eq!(info.logical_cores, 2);
        assert_eq!(info.flags, vec!["avx", "avx2", "ht"]);
    }

    #[test]
    fn missing_cpuinfo_fields_use_fallbacks() {
        let info = CpuInfo::parse("");

        assert_eq!(info.vendor_id, "Unknown");
        assert_eq!(info.model_name, "Unknown");
        assert_eq!(info.physical_cores, 1);
        assert_eq!(info.logical_cores, 0);
        assert!(info.flags.is_empty());
    }

    #[test]
    fn detects_only_numbered_cpu_lines() {
        assert!(is_core_line("cpu0"));
        assert!(is_core_line("cpu12 1 2 3"));
        assert!(!is_core_line("cpu"));
        assert!(!is_core_line("cpuidle"));
    }
}
