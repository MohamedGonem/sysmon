#[cfg(feature = "bat")]
mod bat;

#[cfg(feature = "cpu")]
mod cpu;

#[cfg(feature = "mem")]
mod mem;

#[cfg(feature = "sys")]
mod sys;

mod utils;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "SysMon")]
#[command(version = "0.1")]
#[command(about = "Personal system monitor tool", long_about = None)]
struct Cli {
    ///Show system information
    #[cfg(feature = "sys")]
    #[arg(long = "sys", short = 'S')]
    sys: bool,

    ///Show CPU usage (or full details with -d flag)
    #[cfg(feature = "cpu")]
    #[arg(long = "cpu", short = 'C')]
    cpu: bool,

    ///Show Core usage (or full details with -d flag)
    #[cfg(feature = "cpu")]
    #[arg(long = "cores", short = 'c')]
    core: bool,

    ///Show memory stats
    #[cfg(feature = "mem")]
    #[arg(long = "mem", short = 'm')]
    mem: Option<Mem>,

    ///Show battery level and charging status
    #[cfg(feature = "bat")]
    #[arg(long = "bat", short = 'b')]
    bat: bool,

    ///Output format for your terminal environment
    #[arg(long = "env", short = 'e', default_value_t = Env::Normal)]
    env: Env,

    ///Output detailed information
    #[arg(long = "detailed", short = 'd', default_value_t = false)]
    detailed: bool,
}

#[cfg(feature = "mem")]
#[derive(Clone, ValueEnum)]
enum Mem {
    Memory,
    Swap,
    All,
}

#[derive(Clone, ValueEnum)]
enum Env {
    Tmux,
    Normal,
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Env::Tmux => write!(f, "tmux"),
            Env::Normal => write!(f, "normal"),
        }
    }
}

#[cfg(feature = "mem")]
impl std::fmt::Display for Mem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mem::Memory => write!(f, "memory"),
            Mem::Swap => write!(f, "swap"),
            Mem::All => write!(f, "all"),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let env = match cli.env {
        Env::Tmux => "tmux".to_string(),
        Env::Normal => "normal".to_string(),
    };

    #[cfg(feature = "mem")]
    let mem = match &cli.mem {
        Some(Mem::Memory) => "memory".to_string(),
        Some(Mem::Swap) => "swap".to_string(),
        Some(Mem::All) | None => "all".to_string(),
    };

    #[cfg(feature = "sys")]
    if cli.sys {
        sys::print_sys(env.clone());
    }

    #[cfg(feature = "cpu")]
    if cli.cpu {
        match cli.detailed {
            false => match cli.core {
                true => {
                    cpu::print_cpu_usage(env.clone());
                    cpu::print_core_usage(env.clone());
                }
                false => cpu::print_core_usage(env.clone()),
            },
            true => match cli.core {
                true => cpu::print_cpu_stats(env.clone(), true),
                false => cpu::print_cpu_stats(env.clone(), false),
            },
        }
    }

    #[cfg(feature = "cpu")]
    if cli.core && !cli.cpu {
        match cli.detailed {
            false => cpu::print_core_usage(env.clone()),
            true => cpu::print_cores_stats(env.clone()),
        }
    }

    #[cfg(feature = "mem")]
    if cli.mem.is_some() {
        mem::print_mem(env.clone(), mem.clone(), cli.detailed);
    }

    #[cfg(feature = "bat")]
    if cli.bat {
        bat::print_battery(env.clone());
    }

    let no_option_selected = {
        #[cfg(feature = "sys")]
        {
            !cli.sys
        }
        #[cfg(not(feature = "sys"))]
        {
            true
        }
    } && {
        #[cfg(feature = "cpu")]
        {
            !cli.cpu && !cli.core
        }
        #[cfg(not(feature = "cpu"))]
        {
            true
        }
    } && {
        #[cfg(feature = "mem")]
        {
            cli.mem.is_none()
        }
        #[cfg(not(feature = "mem"))]
        {
            true
        }
    } && {
        #[cfg(feature = "bat")]
        {
            !cli.bat
        }
        #[cfg(not(feature = "bat"))]
        {
            true
        }
    };

    if no_option_selected {
        println!("No option specified. Use --help for usage.")
    }
}
