use super::types::{CPUStat, CoreStat, UsageStat};
use crate::utils::{color, color_based_on_percentage, color_head};

// !!! Should implement traits instead of functions
// Returns display lines for a given usagestat stuct based on mode weather tmux or normal
pub fn color_usagestat(target: UsageStat, tmux: bool) -> Vec<String> {
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

// Returns display lines for a giving corestat struct based on mode weather tmux or normal
pub fn color_corestat(target: CoreStat, tmux: bool, newline: bool) -> Vec<String> {
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
    if !newline {
        let core_stat = vec![core_id, core_temperature, format!("\t\t{}", "-".repeat(10))];
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

// Returns display lines for a given cpustat struct based on mode weather tmux or normal
pub fn color_cpustat(target: CPUStat, tmux: bool) -> Vec<String> {
    let model_name = format!("{} {}", color_head("Model", tmux), target.model_name);
    let physical_cores_count = format!(
        "{} [{}]",
        color_head("Phusical Cores", tmux),
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

    let result = vec![
        model_name,
        physical_cores_count,
        logical_cores_count,
        current_frequency,
        min_frequency,
        max_frequency,
        temperature,
        usage_percentage,
    ];
    [result, usage, core_stats].concat()
}
// -------------------------------------------
// end of display struct type
// -------------------------------------------
