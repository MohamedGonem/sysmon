mod bat;
mod cli;
mod cpu;
mod disk;
mod mem;
mod net;
mod proc;
mod sensors;
mod sys;
mod utils;

use std::process::ExitCode;

use cli::{Cli, ParseOutcome};
use sysmon_core::search::ProcessFilter;

fn main() -> ExitCode {
    restore_sigpipe();

    match Cli::parse_args() {
        Ok(ParseOutcome::Run(cli)) => run(cli),
        Ok(ParseOutcome::Help) => {
            print!("{}", cli::HELP);
            ExitCode::SUCCESS
        }
        Ok(ParseOutcome::Version) => {
            println!("SysMon {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}\n\nFor more information, try '--help'.");
            ExitCode::from(2)
        }
    }
}

fn restore_sigpipe() {
    // SAFETY: setting a process-wide signal disposition to the default has no
    // memory-safety preconditions. Rust ignores SIGPIPE by default, which turns
    // `sysmon-cli | head` into a panic on EPIPE.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn run(cli: Cli) -> ExitCode {
    let tmux = cli.env.is_tmux();
    let process_filter = match cli.search.as_ref() {
        Some(search) => match ProcessFilter::new(&search.pattern, search.mode) {
            Ok(filter) => Some(filter),
            Err(error) => {
                eprintln!("error: invalid search pattern: {error}");
                return ExitCode::from(2);
            }
        },
        None => None,
    };

    if cli.sys {
        sys::print_sys(tmux);
    }

    if cli.cpu {
        match (cli.detailed, cli.core) {
            (false, true) => cpu::print_cpu_usage_and_cores(tmux),
            (false, false) => cpu::print_cpu_usage(tmux),
            (true, cores) => cpu::print_cpu_stats(tmux, cores),
        }
    }

    if cli.core && !cli.cpu {
        match cli.detailed {
            false => cpu::print_core_usage(tmux),
            true => cpu::print_cores_stats(tmux),
        }
    }

    if let Some(mem) = cli.mem {
        mem::print_mem(tmux, mem.as_str(), cli.detailed);
    }

    if cli.bat {
        bat::print_battery(tmux, cli.detailed);
    }

    if cli.net {
        net::print_network(tmux, cli.detailed);
    }

    if cli.disk {
        disk::print_disks(tmux, cli.detailed);
    }

    if cli.process {
        proc::print_processes(tmux, cli.detailed, process_filter.as_ref());
    }

    if cli.sensors {
        sensors::print_sensors(tmux, cli.detailed);
    }

    let no_option_selected = !cli.sys
        && !cli.cpu
        && !cli.core
        && cli.mem.is_none()
        && !cli.bat
        && !cli.net
        && !cli.disk
        && !cli.process
        && cli.search.is_none()
        && !cli.sensors;

    if no_option_selected {
        println!("No option specified. Use --help for usage.")
    }

    ExitCode::SUCCESS
}
