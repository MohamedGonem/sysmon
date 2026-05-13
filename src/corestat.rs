use crate::usagestat::{UsageStat, color_usagestat};
use crate::utils::{color_based_on_percentage, color_head};

pub struct CoreStat {
    pub id: u64,
    pub usage_percentage: f64,
    pub temperature: f64,
    pub usage: UsageStat,
}

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
