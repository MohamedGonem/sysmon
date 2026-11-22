use std::time::Duration;

use crate::utils::{color, color_head, format_bytes, truncate};
use sysmon_core::search::ProcessFilter;
use sysmon_core::sys::format_epoch;
use sysmon_core::types::{ProcessStat, ProcessStats};

const SAMPLE_WINDOW: Duration = Duration::from_secs(1);

pub fn print_processes(tmux: bool, detailed: bool, filter: Option<&ProcessFilter>) {
    match ProcessStats::sample(SAMPLE_WINDOW) {
        Some(stats) => display_processes(stats, tmux, detailed, filter),
        None => eprintln!("error: could not read process statistics"),
    }
}

fn display_processes(
    stats: ProcessStats,
    tmux: bool,
    detailed: bool,
    filter: Option<&ProcessFilter>,
) {
    let current_pid = std::process::id();
    let mut processes: Vec<_> = stats
        .processes
        .into_iter()
        // The search pattern is part of this process's own command line, so
        // it would otherwise always match itself.
        .filter(|process| filter.is_none() || process.pid != current_pid)
        .filter(|process| filter.is_none_or(|filter| filter.is_match(process)))
        .collect();

    if processes.is_empty() {
        match filter {
            Some(filter) => println!("No processes matching '{}'", filter.pattern()),
            None => println!("No processes found"),
        }
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
