mod bat;
mod cpu;
mod mem;
mod utils;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "SysMon")]
#[command(version = "0.1")]
#[command(about = "Personal system monitor tool", long_about = None)]
struct Cli {
    ///Show CPU usage
    #[arg(long = "cpu", short = 'c')]
    cpu: bool,

    ///Show memory stats
    #[arg(long = "mem", short = 'm')]
    mem: Option<Mem>,

    ///Show battery level and charging status
    #[arg(long = "bat", short = 'b')]
    bat: bool,

    ///Output format for your terminal enviroment
    #[arg(long = "env", short = 'e', default_value_t = Env::Normal)]
    env: Env,

    ///Output detailed informatiob
    #[arg(long = "detailed", short = 'd', default_value_t = false)]
    detailed: bool,
}

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

    let mem = match &cli.mem {
        Some(Mem::Memory) => "memory".to_string(),
        Some(Mem::Swap) => "swap".to_string(),
        Some(Mem::All) | None => "all".to_string(),
    };

    if cli.cpu {
        cpu::print_cpu(env.clone());
    }

    if cli.mem.is_some() {
        mem::print_mem(env.clone(), mem.clone(), cli.detailed);
    }

    if cli.bat {
        bat::print_battery(env.clone());
    }

    if !cli.cpu && cli.mem.is_none() && !cli.bat {
        println!("No option specified. Use --help for usage.")
    }
}
