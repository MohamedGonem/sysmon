use std::ffi::OsString;
use std::fmt;
use std::str::FromStr;

use sysmon_core::search::SearchMode;

const MEM_OPTION: &str = "--mem <MEM>";
const MEM_VALUES: &str = "memory, swap, all";
const ENV_OPTION: &str = "--env <ENV>";
const ENV_VALUES: &str = "tmux, normal";
const SEARCH_OPTION: &str = "--search <TEXT>";
const REGEX_OPTION: &str = "--regex <REGEX>";

pub const HELP: &str = "\
Personal system monitor tool

Usage: sysmon-cli [OPTIONS]

Options:
  -S, --sys              Show system information
  -C, --cpu              Show CPU usage (or full details with -d flag)
  -c, --cores            Show Core usage (or full details with -d flag)
  -m, --mem <MEM>        Show memory stats [possible values: memory, swap, all]
  -b, --bat              Show battery level and charging status
  -n, --net              Show network stats
  -D, --disk             Show disk usage (or full details with -d flag)
  -p, --proc             Show process stats (or full details with -d flag)
  -s, --search <TEXT>    Filter processes by text (case-insensitive)
  -r, --regex <REGEX>    Filter processes by regex (case-insensitive)
  -t, --sensors          Show temperatures and fans
  -e, --env <ENV>        Output format for your terminal environment [default: normal] [possible values: tmux, normal]
  -d, --detailed         Output detailed information
  -h, --help             Print help
  -V, --version          Print version
";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Cli {
    pub sys: bool,
    pub cpu: bool,
    pub core: bool,
    pub mem: Option<Mem>,
    pub bat: bool,
    pub net: bool,
    pub disk: bool,
    pub process: bool,
    pub search: Option<Search>,
    pub sensors: bool,
    pub env: Env,
    pub detailed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Search {
    pub pattern: String,
    pub mode: SearchMode,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    Run(Cli),
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedArgument(String),
    MissingValue {
        option: &'static str,
        possible: Option<&'static str>,
    },
    InvalidValue {
        option: &'static str,
        value: String,
        possible: &'static str,
    },
    InvalidUtf8,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedArgument(argument) => {
                write!(f, "unexpected argument '{argument}' found")
            }
            Self::MissingValue { option, possible } => {
                write!(
                    f,
                    "a value is required for '{option}' but none was supplied"
                )?;
                if let Some(possible) = possible {
                    write!(f, "\n  [possible values: {possible}]")?;
                }
                Ok(())
            }
            Self::InvalidValue {
                option,
                value,
                possible,
            } => write!(
                f,
                "invalid value '{value}' for '{option}'\n  [possible values: {possible}]"
            ),
            Self::InvalidUtf8 => write!(f, "arguments must be valid UTF-8"),
        }
    }
}

impl Cli {
    /// Parse the process arguments.
    pub fn parse_args() -> Result<ParseOutcome, ParseError> {
        Self::parse_from(std::env::args_os().skip(1))
    }

    /// Parse arguments from any iterator of OS strings.
    pub fn parse_from<I, S>(args: I) -> Result<ParseOutcome, ParseError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut cli = Self::default();
        let mut args = args.into_iter().map(Into::into);

        while let Some(argument) = args.next() {
            let argument = argument
                .into_string()
                .map_err(|_| ParseError::InvalidUtf8)?;

            match argument.as_str() {
                "-h" | "--help" => return Ok(ParseOutcome::Help),
                "-V" | "--version" => return Ok(ParseOutcome::Version),
                "-S" | "--sys" => cli.sys = true,
                "-C" | "--cpu" => cli.cpu = true,
                "-c" | "--cores" => cli.core = true,
                "-b" | "--bat" => cli.bat = true,
                "-n" | "--net" => cli.net = true,
                "-D" | "--disk" => cli.disk = true,
                "-p" | "--proc" => cli.process = true,
                "-s" | "--search" => {
                    set_search(
                        &mut cli,
                        next_value(&mut args, SEARCH_OPTION, None)?,
                        SearchMode::Literal,
                    );
                }
                "-r" | "--regex" => {
                    set_search(
                        &mut cli,
                        next_value(&mut args, REGEX_OPTION, None)?,
                        SearchMode::Regex,
                    );
                }
                "-t" | "--sensors" => cli.sensors = true,
                "-d" | "--detailed" => cli.detailed = true,
                "-e" | "--env" => {
                    cli.env = parse_env(next_value(&mut args, ENV_OPTION, Some(ENV_VALUES))?)?;
                }
                "-m" | "--mem" => {
                    cli.mem = Some(parse_mem(next_value(
                        &mut args,
                        MEM_OPTION,
                        Some(MEM_VALUES),
                    )?)?);
                }
                "--" => {
                    if let Some(extra) = args.next() {
                        return Err(ParseError::UnexpectedArgument(
                            extra.to_string_lossy().into_owned(),
                        ));
                    }
                }
                _ => {
                    if let Some(value) = argument.strip_prefix("--env=") {
                        cli.env = parse_env(inline_value(value, ENV_OPTION, Some(ENV_VALUES))?)?;
                    } else if let Some(value) = argument.strip_prefix("--mem=") {
                        cli.mem = Some(parse_mem(inline_value(
                            value,
                            MEM_OPTION,
                            Some(MEM_VALUES),
                        )?)?);
                    } else if let Some(value) = argument.strip_prefix("--search=") {
                        set_search(
                            &mut cli,
                            inline_value(value, SEARCH_OPTION, None)?,
                            SearchMode::Literal,
                        );
                    } else if let Some(value) = argument.strip_prefix("--regex=") {
                        set_search(
                            &mut cli,
                            inline_value(value, REGEX_OPTION, None)?,
                            SearchMode::Regex,
                        );
                    } else if let Some(cluster) = argument
                        .strip_prefix('-')
                        .filter(|rest| !rest.is_empty() && !rest.starts_with('-'))
                    {
                        if let Some(outcome) = parse_short_cluster(cluster, &mut cli, &mut args)? {
                            return Ok(outcome);
                        }
                    } else {
                        return Err(ParseError::UnexpectedArgument(argument));
                    }
                }
            }
        }

        Ok(ParseOutcome::Run(cli))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl FromStr for Mem {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "memory" => Ok(Self::Memory),
            "swap" => Ok(Self::Swap),
            "all" => Ok(Self::All),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Env {
    Tmux,
    #[default]
    Normal,
}

impl Env {
    pub fn is_tmux(self) -> bool {
        matches!(self, Self::Tmux)
    }
}

impl FromStr for Env {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "tmux" => Ok(Self::Tmux),
            "normal" => Ok(Self::Normal),
            _ => Err(()),
        }
    }
}

fn parse_mem(value: String) -> Result<Mem, ParseError> {
    match value.parse::<Mem>() {
        Ok(mem) => Ok(mem),
        Err(()) => Err(ParseError::InvalidValue {
            option: MEM_OPTION,
            value,
            possible: MEM_VALUES,
        }),
    }
}

fn parse_env(value: String) -> Result<Env, ParseError> {
    match value.parse::<Env>() {
        Ok(env) => Ok(env),
        Err(()) => Err(ParseError::InvalidValue {
            option: ENV_OPTION,
            value,
            possible: ENV_VALUES,
        }),
    }
}

fn set_search(cli: &mut Cli, pattern: String, mode: SearchMode) {
    cli.search = Some(Search { pattern, mode });
    cli.process = true;
}

fn next_value<I>(
    args: &mut I,
    option: &'static str,
    possible: Option<&'static str>,
) -> Result<String, ParseError>
where
    I: Iterator<Item = OsString>,
{
    let value = args
        .next()
        .ok_or(ParseError::MissingValue { option, possible })?;

    value.into_string().map_err(|_| ParseError::InvalidUtf8)
}

fn inline_value(
    value: &str,
    option: &'static str,
    possible: Option<&'static str>,
) -> Result<String, ParseError> {
    if value.is_empty() {
        Err(ParseError::MissingValue { option, possible })
    } else {
        Ok(value.to_string())
    }
}

fn parse_short_cluster<I>(
    cluster: &str,
    cli: &mut Cli,
    args: &mut I,
) -> Result<Option<ParseOutcome>, ParseError>
where
    I: Iterator<Item = OsString>,
{
    for (index, short) in cluster.char_indices() {
        match short {
            'S' => cli.sys = true,
            'C' => cli.cpu = true,
            'c' => cli.core = true,
            'b' => cli.bat = true,
            'n' => cli.net = true,
            'D' => cli.disk = true,
            'p' => cli.process = true,
            't' => cli.sensors = true,
            'd' => cli.detailed = true,
            'h' => return Ok(Some(ParseOutcome::Help)),
            'V' => return Ok(Some(ParseOutcome::Version)),
            'e' | 'm' | 's' | 'r' => {
                let (option, possible) = match short {
                    'e' => (ENV_OPTION, Some(ENV_VALUES)),
                    'm' => (MEM_OPTION, Some(MEM_VALUES)),
                    's' => (SEARCH_OPTION, None),
                    'r' => (REGEX_OPTION, None),
                    _ => unreachable!(),
                };

                let rest = &cluster[index + short.len_utf8()..];
                let value = if rest.is_empty() {
                    next_value(args, option, possible)?
                } else if let Some(value) = rest.strip_prefix('=') {
                    inline_value(value, option, possible)?
                } else {
                    rest.to_string()
                };

                match short {
                    'e' => cli.env = parse_env(value)?,
                    'm' => cli.mem = Some(parse_mem(value)?),
                    's' => set_search(cli, value, SearchMode::Literal),
                    'r' => set_search(cli, value, SearchMode::Regex),
                    _ => unreachable!(),
                }

                return Ok(None);
            }
            _ => return Err(ParseError::UnexpectedArgument(format!("-{short}"))),
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<ParseOutcome, ParseError> {
        Cli::parse_from(args.iter().copied())
    }

    fn run(args: &[&str]) -> Cli {
        match parse(args) {
            Ok(ParseOutcome::Run(cli)) => cli,
            other => panic!("expected run, got {other:?}"),
        }
    }

    #[test]
    fn defaults_match_previous_behavior() {
        let cli = run(&[]);

        assert!(!cli.sys);
        assert!(!cli.cpu);
        assert!(!cli.core);
        assert_eq!(cli.mem, None);
        assert!(!cli.bat);
        assert!(!cli.net);
        assert!(!cli.disk);
        assert!(!cli.process);
        assert_eq!(cli.search, None);
        assert!(!cli.sensors);
        assert_eq!(cli.env, Env::Normal);
        assert!(!cli.detailed);
    }

    #[test]
    fn parses_long_flags() {
        let cli = run(&[
            "--sys",
            "--cpu",
            "--cores",
            "--bat",
            "--net",
            "--disk",
            "--proc",
            "--sensors",
            "--detailed",
        ]);

        assert!(cli.sys);
        assert!(cli.cpu);
        assert!(cli.core);
        assert!(cli.bat);
        assert!(cli.net);
        assert!(cli.disk);
        assert!(cli.process);
        assert!(cli.sensors);
        assert!(cli.detailed);
    }

    #[test]
    fn parses_short_flags() {
        let cli = run(&["-S", "-C", "-c", "-b", "-n", "-D", "-p", "-t", "-d"]);

        assert!(cli.sys);
        assert!(cli.cpu);
        assert!(cli.core);
        assert!(cli.bat);
        assert!(cli.net);
        assert!(cli.disk);
        assert!(cli.process);
        assert!(cli.sensors);
        assert!(cli.detailed);
    }

    #[test]
    fn parses_short_flag_clusters() {
        let cli = run(&["-SC"]);
        assert!(cli.sys);
        assert!(cli.cpu);

        let cli = run(&["-Cdb"]);
        assert!(cli.cpu);
        assert!(cli.detailed);
        assert!(cli.bat);

        let cli = run(&["-nDpt"]);
        assert!(cli.net);
        assert!(cli.disk);
        assert!(cli.process);
        assert!(cli.sensors);
    }

    #[test]
    fn parses_mem_in_every_supported_form() {
        for args in [
            &["--mem", "swap"][..],
            &["--mem=swap"][..],
            &["-m", "swap"][..],
            &["-mswap"][..],
            &["-m=swap"][..],
        ] {
            assert_eq!(run(args).mem, Some(Mem::Swap), "{args:?}");
        }
    }

    #[test]
    fn parses_env_in_every_supported_form() {
        for args in [
            &["--env", "tmux"][..],
            &["--env=tmux"][..],
            &["-e", "tmux"][..],
            &["-etmux"][..],
            &["-e=tmux"][..],
        ] {
            assert_eq!(run(args).env, Env::Tmux, "{args:?}");
        }
    }

    #[test]
    fn parses_literal_search_in_every_supported_form() {
        for args in [
            &["--search", "firefox"][..],
            &["--search=firefox"][..],
            &["-s", "firefox"][..],
            &["-sfirefox"][..],
            &["-s=firefox"][..],
        ] {
            let cli = run(args);

            assert_eq!(
                cli.search,
                Some(Search {
                    pattern: "firefox".to_string(),
                    mode: SearchMode::Literal,
                }),
                "{args:?}"
            );
            assert!(cli.process, "search should imply --proc: {args:?}");
        }
    }

    #[test]
    fn parses_regex_search_in_every_supported_form() {
        for args in [
            &["--regex", "^fire.*"][..],
            &["--regex=^fire.*"][..],
            &["-r", "^fire.*"][..],
            &["-r^fire.*"][..],
            &["-r=^fire.*"][..],
        ] {
            let cli = run(args);

            assert_eq!(
                cli.search,
                Some(Search {
                    pattern: "^fire.*".to_string(),
                    mode: SearchMode::Regex,
                }),
                "{args:?}"
            );
            assert!(cli.process, "regex search should imply --proc: {args:?}");
        }
    }

    #[test]
    fn later_search_overrides_earlier_ones() {
        let cli = run(&["-s", "fire", "-r", "^fire"]);

        assert_eq!(
            cli.search,
            Some(Search {
                pattern: "^fire".to_string(),
                mode: SearchMode::Regex,
            })
        );
    }

    #[test]
    fn search_works_inside_short_flag_clusters() {
        let cli = run(&["-psfirefox"]);
        assert!(cli.process);
        assert_eq!(
            cli.search,
            Some(Search {
                pattern: "firefox".to_string(),
                mode: SearchMode::Literal,
            })
        );

        let cli = run(&["-dr^fire"]);
        assert!(cli.detailed);
        assert_eq!(
            cli.search,
            Some(Search {
                pattern: "^fire".to_string(),
                mode: SearchMode::Regex,
            })
        );
    }

    #[test]
    fn later_values_override_earlier_ones() {
        let cli = run(&["--mem", "swap", "--mem", "all"]);

        assert_eq!(cli.mem, Some(Mem::All));
    }

    #[test]
    fn help_and_version_short_circuit() {
        assert_eq!(parse(&["--help"]), Ok(ParseOutcome::Help));
        assert_eq!(parse(&["-h"]), Ok(ParseOutcome::Help));
        assert_eq!(parse(&["-Sh"]), Ok(ParseOutcome::Help));
        assert_eq!(parse(&["-h", "--cpu"]), Ok(ParseOutcome::Help));

        assert_eq!(parse(&["--version"]), Ok(ParseOutcome::Version));
        assert_eq!(parse(&["-V"]), Ok(ParseOutcome::Version));
        assert_eq!(parse(&["-SV"]), Ok(ParseOutcome::Version));
    }

    #[test]
    fn missing_values_are_reported() {
        assert_eq!(
            parse(&["--mem"]),
            Err(ParseError::MissingValue {
                option: MEM_OPTION,
                possible: Some(MEM_VALUES),
            })
        );
        assert_eq!(
            parse(&["--mem="]),
            Err(ParseError::MissingValue {
                option: MEM_OPTION,
                possible: Some(MEM_VALUES),
            })
        );
        assert_eq!(
            parse(&["-m="]),
            Err(ParseError::MissingValue {
                option: MEM_OPTION,
                possible: Some(MEM_VALUES),
            })
        );
        assert_eq!(
            parse(&["--env"]),
            Err(ParseError::MissingValue {
                option: ENV_OPTION,
                possible: Some(ENV_VALUES),
            })
        );
        assert_eq!(
            parse(&["--env="]),
            Err(ParseError::MissingValue {
                option: ENV_OPTION,
                possible: Some(ENV_VALUES),
            })
        );
        assert_eq!(
            parse(&["--search"]),
            Err(ParseError::MissingValue {
                option: SEARCH_OPTION,
                possible: None,
            })
        );
        assert_eq!(
            parse(&["--search="]),
            Err(ParseError::MissingValue {
                option: SEARCH_OPTION,
                possible: None,
            })
        );
        assert_eq!(
            parse(&["-s="]),
            Err(ParseError::MissingValue {
                option: SEARCH_OPTION,
                possible: None,
            })
        );
        assert_eq!(
            parse(&["--regex"]),
            Err(ParseError::MissingValue {
                option: REGEX_OPTION,
                possible: None,
            })
        );
        assert_eq!(
            parse(&["--regex="]),
            Err(ParseError::MissingValue {
                option: REGEX_OPTION,
                possible: None,
            })
        );
        assert_eq!(
            parse(&["-r="]),
            Err(ParseError::MissingValue {
                option: REGEX_OPTION,
                possible: None,
            })
        );
    }

    #[test]
    fn invalid_values_are_reported() {
        assert_eq!(
            parse(&["--mem", "nope"]),
            Err(ParseError::InvalidValue {
                option: MEM_OPTION,
                value: "nope".to_string(),
                possible: MEM_VALUES,
            })
        );
        assert_eq!(
            parse(&["--env", "nope"]),
            Err(ParseError::InvalidValue {
                option: ENV_OPTION,
                value: "nope".to_string(),
                possible: ENV_VALUES,
            })
        );
    }

    #[test]
    fn unexpected_arguments_are_reported() {
        assert!(matches!(
            parse(&["--nope"]),
            Err(ParseError::UnexpectedArgument(_))
        ));
        assert!(matches!(
            parse(&["extra"]),
            Err(ParseError::UnexpectedArgument(_))
        ));
        assert!(matches!(
            parse(&["-x"]),
            Err(ParseError::UnexpectedArgument(_))
        ));
        assert!(matches!(
            parse(&["-Cx"]),
            Err(ParseError::UnexpectedArgument(_))
        ));
        assert!(matches!(
            parse(&["--", "--sys"]),
            Err(ParseError::UnexpectedArgument(_))
        ));
    }

    #[test]
    fn double_dash_alone_is_a_no_op() {
        assert_eq!(parse(&["--"]), Ok(ParseOutcome::Run(Cli::default())));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_arguments() {
        use std::os::unix::ffi::OsStringExt;

        let argument = OsString::from_vec(vec![b'-', 0xff]);

        assert_eq!(Cli::parse_from([argument]), Err(ParseError::InvalidUtf8));
    }

    #[test]
    fn help_lists_every_option() {
        for flag in [
            "--sys",
            "--cpu",
            "--cores",
            "--mem",
            "--bat",
            "--net",
            "--disk",
            "--proc",
            "--search",
            "--regex",
            "--sensors",
            "--env",
            "--detailed",
            "--help",
            "--version",
        ] {
            assert!(HELP.contains(flag), "help is missing {flag}");
        }
    }
}
