use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "SysMon")]
#[command(version = "0.1")]
#[command(about = "Personal system monitor tool", long_about = None)]
pub struct Cli {
    ///Show system information
    #[arg(long = "sys", short = 'S')]
    pub sys: bool,

    ///Show CPU usage (or full details with -d flag)
    #[arg(long = "cpu", short = 'C')]
    pub cpu: bool,

    ///Show Core usage (or full details with -d flag)
    #[arg(long = "cores", short = 'c')]
    pub core: bool,

    ///Show memory stats
    #[arg(long = "mem", short = 'm')]
    pub mem: Option<Mem>,

    ///Show battery level and charging status
    #[arg(long = "bat", short = 'b')]
    pub bat: bool,

    ///Output format for your terminal environment
    #[arg(long = "env", short = 'e', default_value_t = Env::Normal)]
    pub env: Env,

    ///Output detailed information
    #[arg(long = "detailed", short = 'd', default_value_t = false)]
    pub detailed: bool,
}

impl Cli {
    /// Parse the command line arguments.
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Mem {
    Memory,
    Swap,
    All,
}

impl Mem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Swap => "swap",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Env {
    Tmux,
    Normal,
}

impl Env {
    pub fn is_tmux(self) -> bool {
        matches!(self, Self::Tmux)
    }
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tmux => write!(f, "tmux"),
            Self::Normal => write!(f, "normal"),
        }
    }
}

impl std::fmt::Display for Mem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
