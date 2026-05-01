use crate::utils::{color, color_based_on_percentage, color_head};
use std::f64;

struct CoreStat {
    id: u64,
    usage_percentage: f64,
    temperature: f64,
    usage: UsageStat,
}

fn color_corestat(target: CoreStat, tmux: bool) -> Vec<String> {
    let core_id = format!(
        "{}[{}]",
        color_head("\tCore", tmux),
        color_based_on_percentage(target.id.to_string(), target.usage_percentage, tmux, false)
    );
    let core_temperature = format!(
        "\t\t{} {:0.1}c",
        color_head("Temperature", tmux),
        target.temperature
    );
    let core_usage = color_usagestat(target.usage, tmux);
    let core_stat = vec![core_id, core_temperature, format!("\t\t{}", "-".repeat(10))];

    [core_stat, core_usage].concat()
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

fn color_usagestat(target: UsageStat, tmux: bool) -> Vec<String> {
    let formatter = |target: u64, text: &str| format!("\t{} {}mb", color_head(text, tmux), target);
    let user = formatter(target.user, "User");
    let nice = formatter(target.nice, "Nice");
    let system = formatter(target.system, "System");
    let idle = formatter(target.idle, "Idle");
    let iowait = formatter(target.iowait, "IOWait");
    let irq = formatter(target.irq, "Irq");
    let softirq = formatter(target.softirq, "SoftIrq");
    let steal = formatter(target.steal, "Steal");
    let guest = formatter(target.guest, "Guest");
    let guest_nice = formatter(target.guest_nice, "Guest-Nice");

    vec![
        user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice,
    ]
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
fn color_cpustat(target: CPUStat, tmux: bool) -> Vec<String> {
    todo!();
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
fn calc_usages(target_refs: &[&str]) -> Vec<f64> {
    target_refs.iter().map(|u| calc_usage(*u)).collect()
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
    let len = cores.len();
    println!("{}\n{}", color_head("[Cores]", tmux), "-".repeat(35));
    for (i, core) in cores.into_iter().enumerate() {
        let colored_core = color_corestat(core, tmux);
        for stat in colored_core {
            println!("{}", stat);
        }
        if i < len - 1 {
            println!("{}", "-".repeat(25));
        }
    }
}

pub fn print_cpu_usage(env: String) {
    let tmux = env == "tmux";
    let usage = calc_usage("cpu");
    println!("CPU {}", color(usage, tmux));
}

