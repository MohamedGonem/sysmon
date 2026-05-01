use crate::utils::{color, color_based_on_percentage, color_head};
use std::f64;

struct MemStat {
    total_mb: u64,
    used_mb: u64,
    free_mb: u64,
    percentage: f64,
}

fn mem_usage() -> (MemStat, MemStat) {
    let content = std::fs::read_to_string("/proc/meminfo").unwrap();

    let mut mem_available = 0u64;
    let mut mem_total = 0u64;
    let mut swap_available = 0u64;
    let mut swap_total = 0u64;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("MemTotal:") => mem_total = parts.next().unwrap().parse().unwrap(),
            Some("MemAvailable:") => mem_available = parts.next().unwrap().parse().unwrap(),
            Some("SwapTotal:") => swap_total = parts.next().unwrap().parse().unwrap(),
            Some("SwapFree:") => swap_available = parts.next().unwrap().parse().unwrap(),
            _ => {}
        }
    }

    let mem_used = mem_total - mem_available;
    let mem_percentage: f64 = (mem_used * 100) as f64 / mem_total as f64;

    let mem_free_mb = mem_available / 1024;
    let mem_used_mb = mem_used / 1024;
    let mem_total_mb = mem_total / 1024;

    let swap_used = swap_total - swap_available;
    let swap_percentage: f64 = (swap_used * 100) as f64 / swap_total as f64;

    let swap_free_mb = swap_available / 1024;
    let swap_used_mb = swap_used / 1024;
    let swap_total_mb = swap_total / 1024;

    (
        MemStat {
            total_mb: mem_total_mb,
            used_mb: mem_used_mb,
            percentage: mem_percentage,
            free_mb: mem_free_mb,
        },
        MemStat {
            total_mb: swap_total_mb,
            used_mb: swap_used_mb,
            percentage: swap_percentage,
            free_mb: swap_free_mb,
        },
    )
}
