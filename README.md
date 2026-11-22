# sysmon

A system monitoring tool.

## Features
- CPU usage, temperature, frequency, load average, and per-core details
- Memory and swap usage, including free, available, buffers, and cache
- Battery capacity, energy/charge, power, voltage, cycles, and health
- Network interfaces, addresses, counters, and transfer speeds
- Disk mounts, space usage, and per-device I/O
- Process CPU/memory usage, details, and text/regex search
- Temperatures and fan speeds from hwmon

## Status

Early stage. Linux is the supported platform.

> [!TIP]
> Contributions are welcome.

## Installation

check rust [installation](rust/INSTALLATION.md)

## Usage
--------
```
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
```

Process search matches the PID, name, user, and command line. The running
monitor excludes itself from search results. `--regex` supports a small
POSIX-style subset (literals, `.`, `*`, `+`, `?`, `{m,n}`, classes, groups,
alternation, anchors, and common escapes).

## License
[MIT](LICENSE)
