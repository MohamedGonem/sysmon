use std::time::Duration;

use crate::utils::{color, color_head, format_frequency};
use sysmon_core::cpu::CpuTimes;
use sysmon_core::types::{CPUStat, CoreStat, UsageStat};

/// Sampling window used by the CLI. Core exposes the primitives; the frontend chooses the policy.
const SAMPLE_WINDOW: Duration = Duration::from_secs(1);

pub fn print_cores_stats(tmux: bool) {
    match CoreStat::sample(SAMPLE_WINDOW) {
        Some(cores) => display_cores_stats(cores, tmux),
        None => eprintln!("error: could not read CPU statistics"),
    }
}

pub fn print_cpu_stats(tmux: bool, cores: bool) {
    match CPUStat::sample(SAMPLE_WINDOW, cores) {
        Some(cpu) => display_cpu_stats(cpu, tmux, cores),
        None => eprintln!("error: could not read CPU statistics"),
    }
}

pub fn print_cpu_usage(tmux: bool) {
    match CpuTimes::sample_usage(SAMPLE_WINDOW) {
        Some(usage) => display_cpu_usage(usage, tmux),
        None => eprintln!("error: could not read CPU statistics"),
    }
}

pub fn print_core_usage(tmux: bool) {
    match CpuTimes::sample_core_usages(SAMPLE_WINDOW) {
        Some(usages) => display_core_usage(usages, tmux),
        None => eprintln!("error: could not read CPU statistics"),
    }
}

pub fn print_cpu_usage_and_cores(tmux: bool) {
    match CpuTimes::sample_usage_and_cores(SAMPLE_WINDOW) {
        Some((usage, cores)) => {
            display_cpu_usage(usage, tmux);
            display_core_usage(cores, tmux);
        }
        None => eprintln!("error: could not read CPU statistics"),
    }
}

// Returns display lines for a given UsageStat struct based on whether the output is for tmux.
fn color_usagestat(target: UsageStat, tmux: bool) -> Vec<String> {
    let formatter =
        |target: u64, text: &str| format!("\t{} {}Tick", color_head(text, tmux), target);
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

// Returns display lines for a given CoreStat struct based on whether the output is for tmux.
fn color_corestat(target: CoreStat, tmux: bool, newline: bool) -> Vec<String> {
    let core_id = format!(
        "{}[{}]",
        color_head("\tCore", tmux),
        color_head(&target.id.to_string(), tmux)
    );
    let core_state = if target.online {
        format!("\t[{}]", color(target.usage_percentage, tmux))
    } else {
        format!("\t[{}]", color_head("offline", tmux))
    };
    let core_temperature = match target.temperature {
        Some(temperature) => format!(
            "\t\t{} [{:0.1} C]",
            color_head("Temperature", tmux),
            temperature
        ),
        None => format!("\t\t{} [N/A]", color_head("Temperature", tmux)),
    };
    let core_frequency = match target.current_frequency {
        Some(frequency) => format!(
            "\t\t{} [{}]",
            color_head("Frequency", tmux),
            format_frequency(frequency)
        ),
        None => format!("\t\t{} [N/A]", color_head("Frequency", tmux)),
    };
    let core_governor = match target.governor {
        Some(governor) => format!("\t\t{} [{}]", color_head("Governor", tmux), governor),
        None => format!("\t\t{} [N/A]", color_head("Governor", tmux)),
    };
    let core_online = format!("\t\t{} [{}]", color_head("Online", tmux), target.online);

    let core_usage = color_usagestat(target.usage, tmux);
    let header = format!("{core_id}{core_state}");

    if !newline {
        let core_stat = vec![
            header,
            core_temperature,
            core_frequency,
            core_governor,
            core_online,
            format!("\t\t{}", "-".repeat(10)),
        ];
        [core_stat, core_usage].concat()
    } else {
        let core_stat = vec![
            "-".repeat(10),
            header,
            core_temperature,
            core_frequency,
            core_governor,
            core_online,
            format!("\t\t{}", "-".repeat(10)),
        ];
        [core_stat, core_usage].concat()
    }
}

fn format_cache_levels(target: Vec<[String; 3]>, tmux: bool) -> Vec<String> {
    if target.is_empty() {
        return Vec::new();
    }

    let mut result = vec![color_head("Cache Info", tmux)];
    result.extend(target.iter().flat_map(|cache| {
        let [cache_type, cache_level, cache_size] = cache;
        let mut lines = vec![
            format!("\t{} [{}]", color_head("[Type]", tmux), cache_type),
            format!("\t{} [{}]", color_head("[Level]", tmux), cache_level),
            format!("\t{} [{}]", color_head("[Size]", tmux), cache_size),
        ];
        lines.push("\t-----".to_string());
        lines
    }));
    result
}

fn format_flags(target: Vec<String>, tmux: bool) -> Vec<String> {
    if target.is_empty() {
        return Vec::new();
    }

    let mut result = vec![color_head("Flags", tmux)];
    result.extend(target.iter().map(|flag| format!("\t[{}]", flag)));
    result
}

// Returns display lines for a given CPUStat struct based on whether the output is for tmux.
fn color_cpustat(target: CPUStat, tmux: bool) -> (Vec<String>, Vec<String>) {
    let model_name = format!("{} {}", color_head("Model", tmux), target.model_name);
    let vendor_id = format!("{} {}", color_head("Vendor Id", tmux), target.vendor_id);
    let governor = format!("{} {}", color_head("Governor", tmux), target.governor);
    let cache_info = format_cache_levels(target.cache_levels, tmux);
    let flags = format_flags(target.flags, tmux);
    let physical_cores_count = format!(
        "{} [{}]",
        color_head("Physical Cores", tmux),
        target.physical_cores_count
    );
    let logical_cores_count = format!(
        "{} [{}]",
        color_head("Logical Cores", tmux),
        target.logical_cores_count
    );
    let current_frequency = format!(
        "{} [{}]",
        color_head("Frequency", tmux),
        format_frequency(target.current_frequency)
    );
    let min_frequency = format!(
        "{} [{}]",
        color_head("Min Frequency", tmux),
        format_frequency(target.min_frequency)
    );
    let max_frequency = format!(
        "{} [{}]",
        color_head("Max Frequency", tmux),
        format_frequency(target.max_frequency)
    );
    let temperature = match target.temperature {
        Some(temperature) => format!(
            "{} [{:0.2} C]",
            color_head("Temperature", tmux),
            temperature
        ),
        None => format!("{} [N/A]", color_head("Temperature", tmux)),
    };
    let load_average = format!(
        "{} [{:.2} {:.2} {:.2}]",
        color_head("Load Average", tmux),
        target.load_average.one,
        target.load_average.five,
        target.load_average.fifteen
    );
    let context_switches = format!(
        "{} [{}]",
        color_head("Context Switches", tmux),
        target.context_switches
    );
    let processes_running = format!(
        "{} [{}]",
        color_head("Processes Running", tmux),
        target.processes_running
    );
    let usage_percentage = format!(
        "{} [{}]",
        color_head("Usage", tmux),
        color(target.usage_percentage, tmux)
    );
    let usage = color_usagestat(target.usage, tmux);

    let core_stats: Vec<_> = target
        .core_stats
        .into_iter()
        .flat_map(|core| color_corestat(core, tmux, true))
        .collect();

    (
        [
            vec![model_name, vendor_id, governor],
            cache_info,
            flags,
            vec![
                physical_cores_count,
                logical_cores_count,
                current_frequency,
                min_frequency,
                max_frequency,
                temperature,
                load_average,
                context_switches,
                processes_running,
                usage_percentage,
            ],
            usage,
        ]
        .concat(),
        core_stats,
    )
}

fn display_cores_stats(cores: Vec<CoreStat>, tmux: bool) {
    let len = cores.len();
    println!("{}\n{}", color_head("[Cores]", tmux), "-".repeat(35));
    for (i, core) in cores.into_iter().enumerate() {
        for stat in color_corestat(core, tmux, false) {
            println!("{stat}");
        }
        if i + 1 < len {
            println!("{}", "-".repeat(25));
        }
    }
}

fn display_cpu_stats(cpu: CPUStat, tmux: bool, show_cores: bool) {
    let (cpu, cores) = color_cpustat(cpu, tmux);
    for stat in cpu {
        println!("{stat}");
    }
    if show_cores {
        for core_stat in cores {
            println!("{core_stat}");
        }
    }
}

fn display_cpu_usage(usage: f64, tmux: bool) {
    println!("CPU {}", color(usage, tmux));
}

fn display_core_usage(usages: Vec<(u64, f64)>, tmux: bool) {
    for (id, usage) in usages {
        println!(
            "{}[{}] {}",
            color_head("Core", tmux),
            color_head(&id.to_string(), tmux),
            color(usage, tmux)
        );
    }
}
