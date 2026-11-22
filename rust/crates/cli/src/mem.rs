use crate::utils::{color, color_based_on_percentage, color_head};
use sysmon_core::types::{MemStat, MemoryStats};

pub fn print_mem(tmux: bool, mode: &str, detailed: bool) {
    let Some(stats) = MemoryStats::read() else {
        eprintln!("error: could not read /proc/meminfo");
        return;
    };

    match mode {
        "swap" => match stats.swap {
            Some(swap) => display_swap(swap, tmux, detailed),
            None => display_no_swap(tmux),
        },
        "memory" => display_mem(stats.memory, tmux, detailed),
        "all" => {
            display_mem(stats.memory, tmux, detailed);
            match stats.swap {
                Some(swap) => display_swap(swap, tmux, detailed),
                None => display_no_swap(tmux),
            }
        }
        _ => println!("Invalid choice"),
    }
}

fn display_data(target: MemStat, target_name: &str, tmux: bool, detailed: bool) {
    let target_head = color_head(target_name, tmux);
    let target_percentage = color(target.percentage, tmux);
    let target_free =
        color_based_on_percentage(target.free_mb.to_string(), target.percentage, tmux, true);
    let target_used =
        color_based_on_percentage(target.used_mb.to_string(), target.percentage, tmux, false);
    let target_total = color_head(&target.total_mb.to_string(), tmux);

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

fn display_mem(target: MemStat, tmux: bool, detailed: bool) {
    display_data(target, "Mem", tmux, detailed);
}

fn display_swap(target: MemStat, tmux: bool, detailed: bool) {
    display_data(target, "Swap", tmux, detailed);
}

fn display_no_swap(tmux: bool) {
    println!("{}: No swap configured", color_head("Swap", tmux));
}
