use std::collections::HashSet;
use std::fs;
use std::time::Duration;

use crate::types::{CPUStat, CoreStat, LoadAverage, UsageStat};

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
    ///
    /// `guest` and `guest_nice` are already included in `user` and `nice` by
    /// the kernel, so they are not added again here.
    pub fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
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

impl LoadAverage {
    /// Read and parse `/proc/loadavg`.
    pub fn read() -> Self {
        fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|content| Self::parse(&content))
            .unwrap_or_default()
    }

    /// Parse the 1, 5 and 15 minute load averages from `/proc/loadavg`.
    pub fn parse(content: &str) -> Option<Self> {
        let mut fields = content.split_whitespace();

        Some(Self {
            one: fields.next()?.parse().ok()?,
            five: fields.next()?.parse().ok()?,
            fifteen: fields.next()?.parse().ok()?,
        })
    }
}

/// A single read of `/proc/stat`, containing CPU counters and system activity counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuTimes {
    pub total: UsageStat,
    pub cores: Vec<UsageStat>,
    pub core_ids: Vec<u64>,
    pub context_switches: u64,
    pub processes_running: u64,
}

impl CpuTimes {
    /// Read and parse `/proc/stat`.
    pub fn read() -> Option<Self> {
        let content = fs::read_to_string("/proc/stat").ok()?;
        Self::parse(&content)
    }

    /// Parse the contents of `/proc/stat`.
    pub fn parse(content: &str) -> Option<Self> {
        let mut total = None;
        let mut cores = Vec::new();
        let mut core_ids = Vec::new();
        let mut context_switches = 0;
        let mut processes_running = 0;

        for line in content.lines() {
            if line.split_whitespace().next() == Some("cpu") {
                total = Some(UsageStat::from_proc_stat_line(line));
            } else if let Some(id) = core_id(line) {
                core_ids.push(id);
                cores.push(UsageStat::from_proc_stat_line(line));
            } else if let Some(value) = line.strip_prefix("ctxt ") {
                context_switches = value.trim().parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("procs_running ") {
                processes_running = value.trim().parse().unwrap_or(0);
            }
        }

        Some(Self {
            total: total?,
            cores,
            core_ids,
            context_switches,
            processes_running,
        })
    }

    /// Aggregate CPU usage percentage between an earlier sample and this sample.
    pub fn usage_since(&self, earlier: &Self) -> f64 {
        self.total.usage_since(&earlier.total)
    }

    /// Per-core CPU usage percentages between an earlier sample and this sample.
    ///
    /// Cores are matched by their `/proc/stat` id so hotplugged CPUs do not shift results.
    pub fn core_usages_since(&self, earlier: &Self) -> Vec<(u64, f64)> {
        self.core_ids
            .iter()
            .zip(&self.cores)
            .map(|(id, later)| {
                let usage = earlier
                    .usage_for(*id)
                    .map_or(0.0, |earlier| later.usage_since(earlier));
                (*id, usage)
            })
            .collect()
    }

    /// Sample aggregate CPU usage over a window.
    pub fn sample_usage(window: Duration) -> Option<f64> {
        let (earlier, later) = Self::sample_pair(window)?;
        Some(later.usage_since(&earlier))
    }

    /// Sample every core's usage over a window.
    pub fn sample_core_usages(window: Duration) -> Option<Vec<(u64, f64)>> {
        let (earlier, later) = Self::sample_pair(window)?;
        Some(later.core_usages_since(&earlier))
    }

    /// Sample aggregate and per-core usage over one shared window.
    pub fn sample_usage_and_cores(window: Duration) -> Option<(f64, Vec<(u64, f64)>)> {
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

    fn usage_for(&self, id: u64) -> Option<&UsageStat> {
        self.core_ids
            .iter()
            .position(|candidate| *candidate == id)
            .map(|index| &self.cores[index])
    }

    fn core_stats_since(&self, earlier: &Self) -> Vec<CoreStat> {
        let mut ids = cpu_sysfs_ids();
        for id in &self.core_ids {
            if !ids.contains(id) {
                ids.push(*id);
            }
        }
        ids.sort_unstable();
        ids.dedup();

        ids.into_iter()
            .map(|id| {
                let later = self.usage_for(id);
                let earlier_usage = earlier.usage_for(id);
                let online = cpu_online(id);
                let usage_percentage = match (online, later, earlier_usage) {
                    (true, Some(later), Some(earlier)) => later.usage_since(earlier),
                    _ => 0.0,
                };

                CoreStat {
                    id,
                    usage_percentage,
                    temperature: online.then(|| core_temperature(id)).flatten(),
                    current_frequency: online.then(|| core_frequency(id)).flatten(),
                    governor: online.then(|| core_governor(id)).flatten(),
                    online,
                    usage: later.copied().unwrap_or_default(),
                }
            })
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
            cache_levels: (0..CACHE_INDEXES)
                .map(cache_level)
                .filter(|cache| !cache[1].is_empty() && !cache[2].is_empty())
                .collect(),
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
            load_average: LoadAverage::read(),
            context_switches: later.context_switches,
            processes_running: later.processes_running,
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
        let content = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let mut info = Self::parse(&content);
        let ids = cpu_sysfs_ids();

        if !ids.is_empty() {
            info.logical_cores = ids.len() as u64;
            if let Some(physical_cores) = physical_core_count(&ids) {
                info.physical_cores = physical_cores;
            }
        }

        info
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

fn core_id(line: &str) -> Option<u64> {
    let rest = line.strip_prefix("cpu")?;
    if !rest.chars().next()?.is_ascii_digit() {
        return None;
    }

    rest.split_whitespace().next()?.parse().ok()
}

fn cpu_sysfs_ids() -> Vec<u64> {
    let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name();
            name.to_str()?.strip_prefix("cpu")?.parse().ok()
        })
        .collect()
}

/// Count unique (package, physical core) pairs exposed by sysfs.
fn physical_core_count(ids: &[u64]) -> Option<u64> {
    let mut pairs = HashSet::new();

    for id in ids {
        let topology = format!("/sys/devices/system/cpu/cpu{id}/topology");
        let Some(package) = read_trimmed(&format!("{topology}/physical_package_id"))
            .and_then(|value| value.parse::<u64>().ok())
        else {
            continue;
        };
        let Some(core) = read_trimmed(&format!("{topology}/core_id"))
            .and_then(|value| value.parse::<u64>().ok())
        else {
            continue;
        };

        pairs.insert((package, core));
    }

    (!pairs.is_empty()).then_some(pairs.len() as u64)
}

fn cpu_online(id: u64) -> bool {
    match read_trimmed(&format!("/sys/devices/system/cpu/cpu{id}/online")) {
        Some(value) => value == "1",
        None => true,
    }
}

fn read_trimmed(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|content| content.trim().to_string())
}

fn read_frequency_opt(path: &str) -> Option<f64> {
    read_trimmed(path)
        .and_then(|value| value.parse::<f64>().ok())
        .map(|khz| khz / 1000.0)
}

fn read_frequency(path: &str) -> f64 {
    read_frequency_opt(path).unwrap_or(0.0)
}

fn core_frequency(id: u64) -> Option<f64> {
    read_frequency_opt(&format!(
        "/sys/devices/system/cpu/cpu{id}/cpufreq/scaling_cur_freq"
    ))
}

fn core_governor(id: u64) -> Option<String> {
    read_trimmed(&format!(
        "/sys/devices/system/cpu/cpu{id}/cpufreq/scaling_governor"
    ))
}

fn governor(id: u64) -> String {
    core_governor(id).unwrap_or_else(|| "Unknown".to_string())
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
        let Ok(name) = fs::read_to_string(format!("/sys/class/hwmon/hwmon{index}/name")) else {
            continue;
        };
        let name = name.trim();
        if name != "coretemp" && name != "k10temp" {
            continue;
        }

        let Ok(raw) = fs::read_to_string(format!("/sys/class/hwmon/hwmon{index}/{input}")) else {
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
    let physical_id = read_trimmed(&format!(
        "/sys/devices/system/cpu/cpu{core_id}/topology/core_id"
    ))
    .and_then(|value| value.parse::<u64>().ok());

    if let Some(physical_id) = physical_id
        && let Some(temperature) = hwmon_core_temperature(physical_id)
    {
        return Some(temperature);
    }

    read_hwmon_temperature(&format!("temp{}_input", core_id + 2))
}

/// Find the `coretemp`/`k10temp` sensor whose label matches a physical core.
///
/// SMT siblings share a physical core, so logical CPUs 4-7 on a 4c/8t part map
/// back to the `Core 0`..`Core 3` sensors instead of looking for missing
/// `temp6_input`..`temp9_input` files.
fn hwmon_core_temperature(core_id: u64) -> Option<f64> {
    for index in 0..HWMON_COUNT {
        let Ok(name) = fs::read_to_string(format!("/sys/class/hwmon/hwmon{index}/name")) else {
            continue;
        };
        let name = name.trim();
        if name != "coretemp" && name != "k10temp" {
            continue;
        }

        let core_label = format!("Core {core_id}");
        let cpu_label = format!("CPU {core_id}");

        for sensor in 1..=HWMON_COUNT {
            let Some(label) =
                read_trimmed(&format!("/sys/class/hwmon/hwmon{index}/temp{sensor}_label"))
            else {
                continue;
            };
            if label != core_label && label != cpu_label {
                continue;
            }

            let Ok(raw) =
                fs::read_to_string(format!("/sys/class/hwmon/hwmon{index}/temp{sensor}_input"))
            else {
                continue;
            };
            if let Ok(millidegrees) = raw.trim().parse::<f64>() {
                return Some(millidegrees / 1000.0);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{CpuInfo, CpuTimes, LoadAverage, core_id};
    use crate::types::UsageStat;

    fn stat(total: u64, idle: u64) -> UsageStat {
        UsageStat {
            user: total.saturating_sub(idle),
            idle,
            ..Default::default()
        }
    }

    #[test]
    fn parses_load_average() {
        let load = LoadAverage::parse("0.52 0.58 0.59 1/1234 5678").unwrap();

        assert_eq!(load.one, 0.52);
        assert_eq!(load.five, 0.58);
        assert_eq!(load.fifteen, 0.59);
    }

    #[test]
    fn returns_none_for_short_load_average() {
        assert!(LoadAverage::parse("0.52 0.58").is_none());
    }

    #[test]
    fn parses_cpu_line_all_fields() {
        let stat = UsageStat::from_proc_stat_line("cpu  1 2 3 4 5 6 7 8 9 10");
        assert_eq!(stat.user, 1);
        assert_eq!(stat.nice, 2);
        assert_eq!(stat.system, 3);
        assert_eq!(stat.idle, 4);
        assert_eq!(stat.iowait, 5);
        assert_eq!(stat.irq, 6);
        assert_eq!(stat.softirq, 7);
        assert_eq!(stat.steal, 8);
        assert_eq!(stat.guest, 9);
        assert_eq!(stat.guest_nice, 10);
    }

    #[test]
    fn parses_core_line() {
        let stat = UsageStat::from_proc_stat_line("cpu0 10 20 30 40 50 60 70 80 90 100");
        assert_eq!(stat.user, 10);
        assert_eq!(stat.idle, 40);
        assert_eq!(stat.iowait, 50);
    }

    #[test]
    fn parses_line_with_extra_whitespace() {
        let stat = UsageStat::from_proc_stat_line("cpu   100 0 50 800 10 0 0 0 0 0");
        assert_eq!(stat.user, 100);
        assert_eq!(stat.system, 50);
        assert_eq!(stat.idle, 800);
    }

    #[test]
    fn parses_empty_line_as_zeros() {
        let stat = UsageStat::from_proc_stat_line("cpu");
        assert_eq!(stat, UsageStat::default());
    }

    #[test]
    fn parses_partial_line_remaining_fields_are_zero() {
        let stat = UsageStat::from_proc_stat_line("cpu 100 200 300");
        assert_eq!(stat.user, 100);
        assert_eq!(stat.nice, 200);
        assert_eq!(stat.system, 300);
        assert_eq!(stat.idle, 0);
        assert_eq!(stat.steal, 0);
    }

    #[test]
    fn parses_non_numeric_field_as_zero() {
        let stat = UsageStat::from_proc_stat_line("cpu 100 bad 300 400 500 600 700 800 900 1000");
        assert_eq!(stat.user, 100);
        assert_eq!(stat.nice, 0);
        assert_eq!(stat.system, 300);
    }

    #[test]
    fn total_excludes_guest_fields_already_in_user_and_nice() {
        let stat = UsageStat {
            user: 10,
            nice: 20,
            system: 30,
            idle: 40,
            iowait: 50,
            irq: 60,
            softirq: 70,
            steal: 80,
            guest: 90,
            guest_nice: 100,
        };

        assert_eq!(stat.total(), 360);
        assert_eq!(stat.idle_total(), 90);
    }

    #[test]
    fn total_all_zeros() {
        let stat = UsageStat::default();
        assert_eq!(stat.total(), 0);
        assert_eq!(stat.idle_total(), 0);
    }

    #[test]
    fn total_only_idle() {
        let stat = stat(100, 100);
        assert_eq!(stat.total(), 100);
        assert_eq!(stat.idle_total(), 100);
    }

    #[test]
    fn total_only_iowait_counts_as_idle() {
        let stat = UsageStat {
            iowait: 50,
            ..Default::default()
        };
        assert_eq!(stat.total(), 50);
        assert_eq!(stat.idle_total(), 50);
    }

    #[test]
    fn usage_is_100_percent_when_nothing_was_idle() {
        assert_eq!(stat(100, 0).usage_since(&stat(0, 0)), 100.0);
    }

    #[test]
    fn usage_is_0_percent_when_everything_was_idle() {
        assert_eq!(stat(100, 100).usage_since(&stat(0, 0)), 0.0);
    }

    #[test]
    fn usage_is_50_percent_when_half_was_idle() {
        assert_eq!(stat(100, 50).usage_since(&stat(0, 0)), 50.0);
    }

    #[test]
    fn usage_uses_only_the_delta() {
        assert_eq!(stat(200, 60).usage_since(&stat(100, 40)), 80.0);
    }

    #[test]
    fn usage_with_large_real_world_values() {
        let earlier = stat(1_000_000, 800_000);
        let later = stat(1_001_000, 800_500);
        assert_eq!(later.usage_since(&earlier), 50.0);
    }

    #[test]
    fn usage_is_zero_when_no_time_passed() {
        let sample = stat(100, 50);
        assert_eq!(sample.usage_since(&sample), 0.0);
    }

    #[test]
    fn full_pipeline_produces_expected_usage() {
        let line1 = "cpu 1000 0 500 8000 500 0 0 0 0 0";
        let line2 = "cpu 1100 0 600 9000 500 0 0 0 0 0";

        let earlier = UsageStat::from_proc_stat_line(line1);
        let later = UsageStat::from_proc_stat_line(line2);

        let expected = 100.0 * (1.0 - 1000.0_f64 / 1200.0_f64);
        assert!((later.usage_since(&earlier) - expected).abs() < 0.001);
    }

    #[test]
    fn parses_aggregate_core_and_activity_counters() {
        let content = "\
cpu  100 0 50 800 10 0 0 0 0 0
cpu0 50 0 25 400 5 0 0 0 0 0
cpu1 50 0 25 400 5 0 0 0 0 0
intr 12345
ctxt 98765
btime 1700000000
processes 1234
procs_running 3
procs_blocked 1
";
        let times = CpuTimes::parse(content).expect("should parse /proc/stat");

        assert_eq!(times.total.user, 100);
        assert_eq!(times.total.idle, 800);
        assert_eq!(times.cores.len(), 2);
        assert_eq!(times.core_ids, vec![0, 1]);
        assert_eq!(times.cores[0].user, 50);
        assert_eq!(times.cores[1].user, 50);
        assert_eq!(times.context_switches, 98765);
        assert_eq!(times.processes_running, 3);
    }

    #[test]
    fn returns_none_without_aggregate_line() {
        assert!(CpuTimes::parse("cpu0 1 2 3 4\n").is_none());
    }

    #[test]
    fn computes_aggregate_and_per_core_usage() {
        let earlier = CpuTimes::parse(
            "\
cpu  100 0 50 800 10 0 0 0 0 0
cpu0 50 0 25 400 5 0 0 0 0 0
cpu1 50 0 25 400 5 0 0 0 0 0
",
        )
        .unwrap();
        let later = CpuTimes::parse(
            "\
cpu  110 0 55 880 10 0 0 0 0 0
cpu0 55 0 27 440 5 0 0 0 0 0
cpu1 55 0 28 440 5 0 0 0 0 0
",
        )
        .unwrap();

        assert!(later.usage_since(&earlier) > 0.0);

        let core_usages = later.core_usages_since(&earlier);
        assert_eq!(core_usages.len(), 2);
        assert_eq!(core_usages[0].0, 0);
        assert_eq!(core_usages[1].0, 1);
        assert!(
            core_usages
                .iter()
                .all(|(_, usage)| (0.0..=100.0).contains(usage))
        );
    }

    #[test]
    fn detects_core_ids() {
        assert_eq!(core_id("cpu0 1 2 3"), Some(0));
        assert_eq!(core_id("cpu12 1 2 3"), Some(12));
        assert_eq!(core_id("cpu 1 2 3"), None);
        assert_eq!(core_id("cpuidle 1 2 3"), None);
    }

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
}
