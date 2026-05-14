use crate::utils::{color, color_based_on_percentage, color_head};
use std::f64;

struct MemStat {
    total_mb: u64,
    used_mb: u64,
    free_mb: u64,
    percentage: f64,
    is_active: Option<bool>,
}

fn mem_usage() -> (MemStat, MemStat) {
    let content = std::fs::read_to_string("/proc/meminfo").unwrap();

    let mut mem_available = 0u64;
    let mut mem_total = 0u64;
    let mut swap_available = 0u64;
    let mut swap_total = 0u64;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("MemTotal:") => mem_total = parts.next().unwrap().parse().unwrap(),
            Some("MemAvailable:") => mem_available = parts.next().unwrap().parse().unwrap(),
            Some("SwapTotal:") => swap_total = parts.next().unwrap().parse().unwrap(),
            Some("SwapFree:") => swap_available = parts.next().unwrap().parse().unwrap(),
            _ => {}
        }
    }

    let mem_used = mem_total - mem_available;
    let mem_percentage: f64 = (mem_used * 100) as f64 / mem_total as f64;

    let mem_free_mb = mem_available / 1024;
    let mem_used_mb = mem_used / 1024;
    let mem_total_mb = mem_total / 1024;

    let swap_state = swap_total != 0;

    let swap_used = if swap_state {
        swap_total - swap_available
    } else {
        0
    };
    let swap_percentage: f64 = if swap_state {
        (swap_used * 100) as f64 / swap_total as f64
    } else {
        0.0
    };

    let swap_free_mb = if swap_state { swap_available / 1024 } else { 0 };
    let swap_used_mb = if swap_state { swap_used / 1024 } else { 0 };
    let swap_total_mb = if swap_state { swap_total / 1024 } else { 0 };

    (
        MemStat {
            total_mb: mem_total_mb,
            used_mb: mem_used_mb,
            percentage: mem_percentage,
            free_mb: mem_free_mb,
            is_active: None,
        },
        MemStat {
            total_mb: swap_total_mb,
            used_mb: swap_used_mb,
            percentage: swap_percentage,
            free_mb: swap_free_mb,
            is_active: Some(swap_state),
        },
    )
}

pub fn print_mem(env: String, mode: String, detailed: bool) {
    let (mem_stat, swap_stat) = mem_usage();
    let tmux = env == "tmux";
    let mem_head = color_head("Mem", tmux);
    let mem_percentage = color(mem_stat.percentage, tmux);
    let mem_free = color_based_on_percentage(
        mem_stat.free_mb.to_string(),
        mem_stat.percentage,
        tmux,
        true,
    );
    let mem_used = color_based_on_percentage(
        mem_stat.used_mb.to_string(),
        mem_stat.percentage,
        tmux,
        false,
    );
    let mem_total = color_head(mem_stat.total_mb.to_string().as_str(), tmux);

    let swap_state = swap_stat.is_active;
    let swap_head = color_head("Swap", tmux);
    let swap_percentage = color(swap_stat.percentage, tmux);
    let swap_free = color_based_on_percentage(
        swap_stat.free_mb.to_string(),
        swap_stat.percentage,
        tmux,
        true,
    );
    let swap_used = color_based_on_percentage(
        swap_stat.used_mb.to_string(),
        swap_stat.percentage,
        tmux,
        false,
    );
    let swap_total = color_head(swap_stat.total_mb.to_string().as_str(), tmux);

    match mode.as_str() {
        "memory" => {
            if !detailed {
                println!("{} {}", mem_head, mem_percentage);
            } else {
                println!("{}:", mem_head);
                println!("\tTotal Memory:\t\t{}mb", mem_total);
                println!("\tTotal Used:\t\t{}mb", mem_used);
                println!("\tTotal Available:\t\t{}mb", mem_free);
                println!("\tUsage:\t\t{}", mem_percentage);
            }
        }
        "swap" => {
            if swap_state == Some(true) {
                if !detailed {
                    println!("{} {}", swap_head, swap_percentage);
                } else {
                    println!("{}:", swap_head);
                    println!("\tTotal Memory:\t\t{}mb", swap_total);
                    println!("\tTotal Used:\t\t{}mb", swap_used);
                    println!("\tTotal Available:\t\t{}mb", swap_free);
                    println!("\tUsage:\t\t{}", swap_percentage);
                }
            } else {
                println!("{}: No swap configured", swap_head);
            }
        }
        "all" => {
            if !detailed {
                println!("{} {}", mem_head, mem_percentage);
                if swap_state == Some(true) {
                    println!("{} {}", swap_head, swap_percentage);
                } else {
                    println!("{}: No swap configured", swap_head);
                }
            } else {
                println!("{}:", mem_head);
                println!("\tTotal Memory:\t\t{}mb", mem_total);
                println!("\tTotal Used:\t\t{}mb", mem_used);
                println!("\tTotal Available:\t\t{}mb", mem_free);
                println!("\tUsage:\t\t{}", mem_percentage);
                if swap_state == Some(true) {
                    println!();
                    println!("{}:", swap_head);
                    println!("\tTotal Memory:\t\t{}mb", swap_total);
                    println!("\tTotal Used:\t\t{}mb", swap_used);
                    println!("\tTotal Available:\t\t{}mb", swap_free);
                    println!("\tUsage:\t\t{}", swap_percentage);
                } else {
                    println!("{}: No swap configured", swap_head);
                }
            }
        }
        _ => {}
    }
}
