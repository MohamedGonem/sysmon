use crate::corestat::{CoreStat, color_corestat};
use crate::usagestat::{UsageStat, color_usagestat};
use crate::utils::{color, color_head};

pub struct CPUStat {
    pub model_name: String,
    pub physical_cores_count: u64,
    pub logical_cores_count: u64,
    pub current_frequency: f64,
    pub min_frequency: f64,
    pub max_frequency: f64,
    pub temperature: f64,
    pub usage_percentage: f64,
    pub core_stats: Vec<CoreStat>,
    pub usage: UsageStat,
}

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
