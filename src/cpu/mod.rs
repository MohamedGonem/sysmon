mod display;
mod io;
mod parser;
#[cfg(test)]
mod tests;
mod types;
use display::{display_core_usage, display_cores_stats, display_cpu_stats, display_cpu_usage};
use io::*;
use parser::{get_total_from_stat, merge_totals, parse_stat_line};
use std::f64;
use types::{CPUStat, CoreStat, UsageStat};

fn get_cpu_or_core_stats(target: &str) -> Option<UsageStat> {
    Some(parse_stat_line(&read_usage_line(target)?))
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

fn core_full_stats() -> Vec<CoreStat> {
    let logical_cores = get_logical_cores();
    let targets: Vec<String> = (0..logical_cores).map(|i| format!("cpu{}", i)).collect();
    let target_refs: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();

    // sample all at once
    let totals_1: Vec<_> = target_refs.iter().map(|t| get_total(t)).collect();
    std::thread::sleep(std::time::Duration::from_millis(1000)); // one sleep only
    let totals_2: Vec<_> = target_refs.iter().map(|t| get_total(t)).collect();

    (0..logical_cores as usize)
        .map(|i| {
            let target = &targets[i];
            CoreStat {
                id: i as u64,
                usage_percentage: merge_totals(totals_1[i], totals_2[i]),
                temperature: get_core_temperature(i as u64),
                usage: get_cpu_or_core_stats(target).unwrap_or(UsageStat {
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
