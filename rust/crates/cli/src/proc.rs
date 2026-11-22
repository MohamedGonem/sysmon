use std::time::Duration;

use crate::utils::{color, color_head, format_bytes, truncate};
use sysmon_core::sys::format_epoch;
use sysmon_core::types::{ProcessStat, ProcessStats};

const SAMPLE_WINDOW: Duration = Duration::from_secs(1);

pub fn print_processes(tmux: bool, detailed: bool) {
    match ProcessStats::sample(SAMPLE_WINDOW) {
        Some(stats) => display_processes(stats, tmux, detailed),
        None => eprintln!("error: could not read process statistics"),
    }
}

fn display_processes(stats: ProcessStats, tmux: bool, detailed: bool) {
    let mut processes = stats.processes;
    if processes.is_empty() {
        println!("No processes found");
        return;
    }

    processes.sort_by(|left, right| {
        right
            .cpu_usage_percentage
            .total_cmp(&left.cpu_usage_percentage)
            .then_with(|| right.memory_percentage.total_cmp(&left.memory_percentage))
    });

    if !detailed {
        println!("{:<7} {:<24} {:>7} {:>7}", "PID", "NAME", "CPU%", "MEM%");

        for process in processes.iter().take(10) {
            println!(
                "{:<7} {:<24} {:>7} {:>7}",
                process.pid,
                truncate(&process.name, 24),
                format!("{:.1}", process.cpu_usage_percentage),
                format!("{:.1}", process.memory_percentage)
            );
        }
        return;
    }

    for process in processes {
        display_process(process, tmux);
    }
}

fn display_process(process: ProcessStat, tmux: bool) {
    let open_fds = process
        .open_fds
        .map(|count| count.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    println!(
        "{} {} ({}):",
        color_head("PID", tmux),
        process.pid,
        process.name
    );
    println!("\tState:\t\t{}", process.state);
    println!("\tCPU:\t\t{}", color(process.cpu_usage_percentage, tmux));
    println!(
        "\tMemory RSS:\t{} ({})",
        format_bytes(process.memory_rss_bytes),
        color(process.memory_percentage, tmux)
    );
    println!("\tVirtual:\t{}", format_bytes(process.virtual_memory_bytes));
    println!("\tUser:\t\t{}", process.user);
    println!("\tThreads:\t{}", process.threads);
    println!("\tOpen FDs:\t{open_fds}");
    println!("\tParent PID:\t{}", process.parent_pid);
    println!("\tNice:\t\t{}", process.nice);
    println!("\tStarted:\t{}", format_epoch(process.start_time));
    println!("\tCommand:\t{}", process.command_line);
}
