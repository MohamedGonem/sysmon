mod display;
mod parser;
mod types;
use display::{display_core_usage, display_cores_stats, display_cpu_stats, display_cpu_usage};
use parser::{get_total_from_stat, merge_totals, parse_stat_line};
use std::f64;
use types::{CPUStat, CoreStat, UsageStat};

fn get_cpu_or_core_stats(target: &str) -> Option<UsageStat> {
    let full_stats = std::fs::read_to_string("/proc/stat").ok()?;
    let line = full_stats.lines().find(|l| {
        l.starts_with(target)
            && l[target.len()..].starts_with(|c: char| c == ' ' || !c.is_ascii_digit())
    })?;
    Some(parse_stat_line(line))
}

fn get_total(target: &str) -> (u64, u64) {
    match get_cpu_or_core_stats(target) {
        Some(s) => get_total_from_stat(s),
        None => (0, 0),
    }
}

fn calc_usage(target: &str) -> f64 {
    let total_1 = get_total(target);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    let total_2 = get_total(target);
    merge_totals(total_1, total_2)
}

fn calc_usages(target_refs: &[&str]) -> Vec<f64> {
    let totals = target_refs.iter().map(|u| get_total(u)).collect::<Vec<_>>();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    core::iter::zip(target_refs, totals)
        .map(|(target, totals_1)| merge_totals(totals_1, get_total(target)))
        .collect()
}

fn get_cpu_temperature() -> Option<f64> {
    for i in 0..10 {
        let name_path = format!("/sys/class/hwmon/hwmon{}/name", i);
        let name = std::fs::read_to_string(&name_path).ok()?;
        let name = name.trim();

        if name == "coretemp" || name == "k10temp" {
            let temp_path = format!("/sys/class/hwmon/hwmon{}/temp1_input", i);
            let raw = std::fs::read_to_string(&temp_path).ok()?;
            let millidegrees: f64 = raw.trim().parse().ok()?;
            return Some(millidegrees / 1000.0);
        }
    }
    let raw = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok()?;
    let millidegrees: f64 = raw.trim().parse().ok()?;
    Some(millidegrees / 1000.0)
}

fn get_core_temperature(core_id: u64) -> Option<f64> {
    for i in 0..10 {
        let name_path = format!("/sys/class/hwmon/hwmon{}/name", i);
        let name = std::fs::read_to_string(&name_path).ok()?;
        let name = name.trim();

        if name == "coretemp" || name == "k10temp" {
            let temp_path = format!("/sys/class/hwmon/hwmon{}/temp{}_input", i, core_id + 2);
            let raw = std::fs::read_to_string(&temp_path).ok()?;
            let millidegree: f64 = raw.trim().parse().ok()?;
            return Some(millidegree / 1000.0);
        }
    }
    None
}

fn get_cpu_model_name() -> String {
    let info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    info.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_physical_cores() -> u64 {
    let info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    info.lines()
        .find(|l| l.starts_with("cpu cores"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1)
}

fn get_logical_cores() -> u64 {
    let info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    info.lines().filter(|l| l.starts_with("processor")).count() as u64
}
fn get_frequency(path: &str) -> f64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map(|khz| khz / 1000.0)
        .unwrap_or(0.0)
}

fn core_full_stats() -> Vec<CoreStat> {
    let logical_cores = get_logical_cores();
    (0..logical_cores)
        .map(|i| {
            let target = format!("cpu{}", i);
            CoreStat {
                id: i,
                usage_percentage: calc_usage(&target),
                temperature: get_core_temperature(i).unwrap_or(0.0),
                usage: get_cpu_or_core_stats(&target).unwrap_or(UsageStat {
                    user: 0,
                    nice: 0,
                    system: 0,
                    idle: 0,
                    iowait: 0,
                    irq: 0,
                    softirq: 0,
                    steal: 0,
                    guest: 0,
                    guest_nice: 0,
                }),
            }
        })
        .collect()
}

fn cpu_full_stats() -> CPUStat {
    CPUStat {
        model_name: get_cpu_model_name(),
        physical_cores_count: get_physical_cores(),
        logical_cores_count: get_logical_cores(),
        current_frequency: get_frequency("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq"),
        min_frequency: get_frequency("/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq"),
        max_frequency: get_frequency("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq"),
        temperature: get_cpu_temperature().unwrap_or(0.0),
        usage_percentage: calc_usage("cpu"),
        usage: get_cpu_or_core_stats("cpu").unwrap_or(UsageStat {
            user: 0,
            nice: 0,
            system: 0,
            idle: 0,
            iowait: 0,
            irq: 0,
            softirq: 0,
            steal: 0,
            guest: 0,
            guest_nice: 0,
        }),
        core_stats: core_full_stats(),
    }
}

pub fn print_cores_stats(env: String) {
    let tmux = env == "tmux";
    let cores = core_full_stats();
    display_cores_stats(cores, tmux);
}

pub fn print_cpu_stats(env: String) {
    let tmux = env == "tmux";
    let cpu = cpu_full_stats();
    display_cpu_stats(cpu, tmux);
}

pub fn print_cpu_usage(env: String) {
    let tmux = env == "tmux";
    let usage = calc_usage("cpu");
    display_cpu_usage(usage, tmux);
}

pub fn print_core_usage(env: String) {
    let tmux = env == "tmux";
    let logical_cores = get_logical_cores();
    let targets: Vec<String> = (0..logical_cores).map(|i| format!("cpu{}", i)).collect();
    let target_refs: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();
    let usages = calc_usages(&target_refs);

    display_core_usage(usages, tmux);
}
