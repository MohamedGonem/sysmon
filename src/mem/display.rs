use crate::utils::{color, color_based_on_percentage, color_head};

fn display_data(target: super::types::MemStat, target_name: &str, tmux: bool, detailed: bool) {
    let target_head = color_head(target_name, tmux);
    let target_percentage = color(target.percentage, tmux);
    let target_free =
        color_based_on_percentage(target.free_mb.to_string(), target.percentage, tmux, true);
    let target_used =
        color_based_on_percentage(target.used_mb.to_string(), target.percentage, tmux, false);
    let target_total = color_head(target.total_mb.to_string().as_str(), tmux);

    if !detailed {
        println!("{} {}", target_head, target_percentage);
    } else {
        println!("{}:", target_head);
        println!("\tTotal Memory:\t\t{}mb", target_total);
        println!("\tTotal Used:\t\t{}mb", target_used);
        println!("\tTotal Available:\t\t{}mb", target_free);
        println!("\tUsage:\t\t{}", target_percentage);
    }
}

pub fn display_mem(target: super::types::MemStat, tmux: bool, detailed: bool) {
    display_data(target, "Mem", tmux, detailed);
}

pub fn display_swap(target: super::types::MemStat, tmux: bool, detailed: bool) {
    match target.is_active {
        Some(true) => display_data(target, "Swap", tmux, detailed),
        _ => println!("{}: No swap configured", color_head("Swap", tmux)),
    }
}
