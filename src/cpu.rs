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
