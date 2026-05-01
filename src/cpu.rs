use std::f64;

struct CoreStat {
    id: u64,
    usage_percentage: f64,
    temperature: f64,
    usage: UsageStat,
}
struct UsageStat {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
    guest: u64,
    guest_nice: u64,
}
struct CPUStat {
    model_name: String,
    physical_cores_count: u64,
    logical_cores_count: u64,
    current_frequency: f64,
    min_frequency: f64,
    max_frequency: f64,
    temperature: f64,
    usage_percentage: f64,
    core_stats: Vec<CoreStat>,
    usage: UsageStat,
}
fn get_cpu_or_core_stats(target: &str) -> Option<UsageStat> {
    let full_stats = std::fs::read_to_string("/proc/stat").ok()?;
    let line = full_stats.lines().find(|l| {
        l.starts_with(target)
            && l[target.len()..].starts_with(|c: char| c == ' ' || !c.is_ascii_digit())
    })?;
    let mut nums = line
        .split_whitespace()
        .skip(1)
        .map(|n| n.parse::<u64>().unwrap_or(0));
    let stats = UsageStat {
        user: nums.next().unwrap_or(0),
        nice: nums.next().unwrap_or(0),
        system: nums.next().unwrap_or(0),
        idle: nums.next().unwrap_or(0),
        iowait: nums.next().unwrap_or(0),
        irq: nums.next().unwrap_or(0),
        softirq: nums.next().unwrap_or(0),
        steal: nums.next().unwrap_or(0),
        guest: nums.next().unwrap_or(0),
        guest_nice: nums.next().unwrap_or(0),
    };
    Some(stats)
}
fn calc_usage(target: &str) -> f64 {
    let get_total = |target: &str| -> (u64, u64) {
        let stats = match get_cpu_or_core_stats(target) {
            Some(s) => s,
            None => return (0, 0),
        };
        let total = stats.user
            + stats.nice
            + stats.system
            + stats.idle
            + stats.iowait
            + stats.irq
            + stats.softirq
            + stats.guest
            + stats.guest_nice
            + stats.steal;
        let idle_total = stats.idle + stats.iowait;
        (total, idle_total)
    };
    let (total1, idle_total1) = get_total(target);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    let (total2, idle_total2) = get_total(target);

    let total = total2 - total1;
    let idle_total = idle_total2 - idle_total1;

    let usage: f64 = 100.0 * (1.0 - idle_total as f64 / total as f64);

    usage
}
fn get_cpu_model_name() -> String {
    let info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    info.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}
