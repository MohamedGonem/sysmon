# sysmon

A Linux system monitor written in Rust.

## Features
- CPU usage, temperature, frequency

## Installation

### From binary (Linux x86_64)
download from releases page
chmod +x sysmon
sudo mv sysmon /usr/local/bin/

### From source
git clone https://github.com/MohamedGonem/sysmon
cd sysmon
cargo build --release
sudo mv target/release/sysmon /usr/local/bin/

## Usage
sysmon -c              # all cores percentage
sysmon -C             # cpu percentage
sysmon -c -d           # all cores detailed
sysmon -C -d           # cpy detailed

## Roadmap
- [ ] memory module
- [ ] disk module
- [ ] network module
- [ ] battery module
- [ ] processes module
- [ ] TUI

## License
MIT
