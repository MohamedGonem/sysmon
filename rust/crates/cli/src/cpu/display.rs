use super::types::{CPUStat, CoreStat, UsageStat};
use crate::utils::{color, color_based_on_percentage, color_head};

// !!! Should implement traits instead of functions
// Returns display lines for a given usagestat stuct based on mode weather tmux or normal
pub fn color_usagestat(target: UsageStat, tmux: bool) -> Vec<String> {
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

// Returns display lines for a giving corestat struct based on mode weather tmux or normal
pub fn color_corestat(target: CoreStat, tmux: bool, newline: bool) -> Vec<String> {
    let core_id = format!(
        "{}[{}]",
        color_head("\tCore", tmux),
        color_based_on_percentage(target.id.to_string(), target.usage_percentage, tmux, false)
    );
    let core_temperature = match target.temperature {
        Some(t) => format!("\t\t{} {:0.1}c", color_head("Temperature", tmux), t),
        None => format!("\t\t{} N/A", color_head("Temperature", tmux)),
    };
    let core_usage_percentage = format!("\t[{}]", color(target.usage_percentage.to_owned(), tmux));

    let core_usage = color_usagestat(target.usage, tmux);
    if !newline {
        let core_stat = vec![
            core_id,
            core_usage_percentage,
            core_temperature,
            format!("\t\t{}", "-".repeat(10)),
        ];
        [core_stat, core_usage].concat()
    } else {
        let core_stat = vec![
            format!("{}", "-".repeat(10)),
            core_id,
            core_temperature,
            format!("\t\t{}", "-".repeat(10)),
        ];
        [core_stat, core_usage].concat()
    }
}

fn format_cache_levels(target: Vec<[String; 3]>, tmux: bool) -> Vec<String> {
    let mut result = vec![format!("{}", color_head("Cache Info", tmux))];
    result.extend(target.iter().flat_map(|cache| {
        let [cache_type, cache_level, cache_size] = cache;
        let mut v = vec![
            format!("\t{} [{}]", color_head("[Type]", tmux), cache_type),
            format!("\t{} [{}]", color_head("[Level]", tmux), cache_level),
            format!("\t{} [{}]", color_head("[Size]", tmux), cache_size),
        ];
        v.push("\t-----".to_string());
        v
    }));
    result
}

fn format_flags(target: Option<Vec<String>>, tmux: bool) -> Vec<String> {
    match target {
        Some(f) => {
            let mut result = vec![format!("{}", color_head("Flags", tmux))];
            result.extend(f.iter().map(|flag| format!("\t[{}]", flag)));
            result
        }
        None => vec![],
    }
}
// Returns display lines for a given cpustat struct based on mode weather tmux or normal
pub fn color_cpustat(target: CPUStat, tmux: bool) -> (Vec<String>, Vec<String>) {
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
        "{} [{:0.1}MHZ]",
        color_head("Frequency", tmux),
        target.current_frequency
    );
    let min_frequency = format!(
        "{} [{:0.1}MHZ]",
        color_head("Min Frequency", tmux),
        target.min_frequency
    );
    let max_frequency = format!(
        "{} [{:0.1}MHZ]",
        color_head("Max Frequency", tmux),
        target.max_frequency
    );
    let temperature = format!(
        "{} [{:0.2}c]",
        color_head("Temperature", tmux),
        target.temperature
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
        .flat_map(|c| color_corestat(c, tmux, true))
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
                usage_percentage,
            ],
            usage,
        ]
        .concat(),
        core_stats,
    )
}
// -------------------------------------------
// end of display struct type
// -------------------------------------------

// ! need comment
pub fn display_cores_stats(cores: Vec<CoreStat>, tmux: bool) {
    let len = cores.len();
    println!("{}\n{}", color_head("[Cores]", tmux), "-".repeat(35));
    for (i, core) in cores.into_iter().enumerate() {
        let colored_core = color_corestat(core, tmux, false);
        for stat in colored_core {
            println!("{}", stat);
        }
        if i < len - 1 {
            println!("{}", "-".repeat(25));
        }
    }
}

// ! need comment
pub fn display_cpu_stats(cpu: CPUStat, tmux: bool, show_cores: bool) {
    let (cpu, cores) = color_cpustat(cpu, tmux);
    for stat in cpu.into_iter() {
        println!("{stat}");
    }
    if show_cores {
        for core_stat in cores.into_iter() {
            println!("{core_stat}")
        }
    }
}

// ! need comment
pub fn display_cpu_usage(usage: f64, tmux: bool) {
    println!("CPU {}", color(usage, tmux));
}

// ! need comment
pub fn display_core_usage(usages: Vec<f64>, tmux: bool) {
    for (i, usage) in usages.iter().enumerate() {
        println!(
            "{}[{}] {}",
            color_head("Core", tmux),
            color_based_on_percentage(i.to_string(), usage.to_owned(), tmux, false),
            color(usage.to_owned(), tmux)
        );
    }
}

// -------------------------------
// end of display helper functions
// -------------------------------
