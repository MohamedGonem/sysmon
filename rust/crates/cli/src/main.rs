mod bat;
mod cli;
mod cpu;
mod mem;
mod sys;
mod utils;

use cli::Cli;

fn main() {
    let cli = Cli::parse_args();
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
