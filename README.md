# sysmon

A system monitoring tool.

## Features
- CPU usage, temperature, frequency
- cores usages
- battery percentage and charging status
- memory (and swap) usage percentage and details

## Status

Early stage. Known Issues:
- Not handling edge-cases well

> [!TIP]
> Contributions are welcome.

## Installation

check rust [installation](rust/INSTALLATION.md)

## Usage
---------
```
Usage: sysmon [OPTIONS]
Options:
  -S, --sys        Show system information
  -C, --cpu        Show CPU usage (or full details with -d flag)
  -c, --cores      Show Core usage (or full details with -d flag)
  -m, --mem <MEM>  Show memory stats [possible values: memory, swap, all]
  -b, --bat        Show battery level and charging status
  -e, --env <ENV>  Output format for your terminal environment [default: normal] [possible values: tmux, normal]
  -d, --detailed   Output detailed information
  -h, --help       Print help
  -V, --version    Print version
```

## License
[MIT](LINSE)
