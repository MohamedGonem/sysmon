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

fn display_mem(target: MemStat, tmux: bool, detailed: bool) {
    let target_head = color_head("Mem", tmux);
    let target_percentage = color(target.percentage, tmux);

    if !detailed {
        println!("{} {}", target_head, target_percentage);
        return;
    }

    let target_total = color_head(&format!("{}mb", target.total_mb), tmux);
    let target_used = color_amount(target.used_mb, target.percentage, tmux);
    let free_percentage = percentage_of(target.free_mb, target.total_mb);
    let target_free = color_amount(target.free_mb, 100.0 - free_percentage, tmux);
    let available_percentage = percentage_of(target.available_mb, target.total_mb);
    let target_available = color_amount(target.available_mb, 100.0 - available_percentage, tmux);
    let target_buffers = color_head(&format!("{}mb", target.buffers_mb), tmux);
    let target_cached = color_head(&format!("{}mb", target.cached_mb), tmux);

    println!("{}:", target_head);
    println!("\tTotal Memory:\t\t{target_total}");
    println!("\tTotal Used:\t\t{target_used}");
    println!("\tFree:\t\t\t{target_free}");
    println!("\tAvailable:\t\t{target_available}");
    println!("\tBuffers:\t\t{target_buffers}");
    println!("\tCached:\t\t\t{target_cached}");
    println!("\tUsage:\t\t\t{target_percentage}");
}

fn display_swap(target: MemStat, tmux: bool, detailed: bool) {
    let target_head = color_head("Swap", tmux);
    let target_percentage = color(target.percentage, tmux);

    if !detailed {
        println!("{} {}", target_head, target_percentage);
        return;
    }

    let target_total = color_head(&format!("{}mb", target.total_mb), tmux);
    let target_used = color_amount(target.used_mb, target.percentage, tmux);
    let free_percentage = percentage_of(target.free_mb, target.total_mb);
    let target_free = color_amount(target.free_mb, 100.0 - free_percentage, tmux);
    let available_percentage = percentage_of(target.available_mb, target.total_mb);
    let target_available = color_amount(target.available_mb, 100.0 - available_percentage, tmux);

    println!("{}:", target_head);
    println!("\tTotal Swap:\t\t{target_total}");
    println!("\tTotal Used:\t\t{target_used}");
    println!("\tFree:\t\t\t{target_free}");
    println!("\tAvailable:\t\t{target_available}");
    println!("\tUsage:\t\t\t{target_percentage}");
}

fn display_no_swap(tmux: bool) {
    println!("{}: No swap configured", color_head("Swap", tmux));
}

fn percentage_of(value_mb: u64, total_mb: u64) -> f64 {
    if total_mb == 0 {
        0.0
    } else {
        value_mb as f64 * 100.0 / total_mb as f64
    }
}

fn color_amount(value_mb: u64, percentage: f64, tmux: bool) -> String {
    color_based_on_percentage(format!("{value_mb}mb"), percentage, tmux)
}
