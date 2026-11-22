use super::types::UsageStat;

pub fn parse_stat_line(line: &str) -> UsageStat {
    let mut nums = line
        .split_whitespace()
        .skip(1)
        .map(|n| n.parse::<u64>().unwrap_or(0));
    UsageStat {
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
    }
}

pub fn get_total_from_stat(stats: UsageStat) -> (u64, u64) {
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
}

pub fn merge_totals((total1, idle_total1): (u64, u64), (total2, idle_total2): (u64, u64)) -> f64 {
    let total = total2 - total1;
    let idle_total = idle_total2 - idle_total1;

    100.0 * (1.0 - idle_total as f64 / total as f64)
}
