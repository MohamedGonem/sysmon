mod bat;
mod cli;
mod cpu;
mod mem;
mod sys;
mod utils;

use std::process::ExitCode;

use cli::{Cli, ParseOutcome};

fn main() -> ExitCode {
    match Cli::parse_args() {
        Ok(ParseOutcome::Run(cli)) => {
            run(cli);
            ExitCode::SUCCESS
        }
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

fn run(cli: Cli) {
    let tmux = cli.env.is_tmux();

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
        bat::print_battery(tmux);
    }

    let no_option_selected = !cli.sys && !cli.cpu && !cli.core && cli.mem.is_none() && !cli.bat;

    if no_option_selected {
        println!("No option specified. Use --help for usage.")
    }
}
