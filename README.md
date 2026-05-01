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
CPU
---
[ ] model name
[ ] vendor id
[ ] cache size
[ ] physical cores count
[ ] logical cores count
[ ] min frequency
[ ] max frequency
[ ] temperature
[ ] usage percentage
[ ] load avg 1min
[ ] load avg 5min
[ ] load avg 15min
[ ] context switches
[ ] procs running
[ ] boot time (btime from /proc/stat)
[ ] per core:
    [ ] id
    [ ] usage percentage
    [ ] temperature
    [ ] current frequency
    [ ] governor
    [ ] online bool
    [ ] user
    [ ] nice
    [ ] system
    [ ] idle
    [ ] iowait
    [ ] irq
    [ ] softirq
    [ ] steal
    [ ] guest
    [ ] guest_nice

MEMORY & SWAP
-------------
[ ] total ram
[ ] used ram
[ ] free ram
[ ] available ram
[ ] buffers
[ ] cached
[ ] total swap
[ ] used swap
[ ] free swap

SYSTEM INFO
-----------
[ ] hostname
[ ] kernel version
[ ] uptime (raw seconds + formatted d/h/m/s)
[ ] distro name (/etc/os-release)
[ ] architecture
[ ] boot time

BATTERY
-------
[ ] status (Charging/Discharging/Full)
[ ] capacity percent
[ ] energy now
[ ] energy full
[ ] energy full design
[ ] power draw (watts)
[ ] voltage
[ ] cycle count
[ ] health percent (energy_full / energy_full_design * 100)

NETWORK
-------
[ ] per interface:
    [ ] name
    [ ] bytes received total
    [ ] bytes sent total
    [ ] download speed (delta)
    [ ] upload speed (delta)
    [ ] packets received
    [ ] packets sent
    [ ] errors in
    [ ] errors out
    [ ] drop in
    [ ] drop out
    [ ] is up (operstate)
    [ ] mac address
    [ ] ip address

DISK
----
[ ] per device:
    [ ] name
    [ ] mount point
    [ ] filesystem type
    [ ] total space
    [ ] used space
    [ ] free space
    [ ] usage percent
    [ ] reads completed
    [ ] writes completed
    [ ] read speed (delta)
    [ ] write speed (delta)
    [ ] is rotational (HDD vs SSD)

PROCESSES
---------
[ ] per process:
    [ ] pid
    [ ] name
    [ ] state
    [ ] cpu usage percent (delta)
    [ ] memory rss
    [ ] memory percent
    [ ] virtual memory size
    [ ] user/owner
    [ ] threads count
    [ ] open file descriptors count
    [ ] command line
    [ ] parent pid
    [ ] nice value
    [ ] start time

TEMPERATURES & FANS
-------------------
[ ] cpu temperature (already done)
[ ] per core temperature (already done)
[ ] disk temperatures
[ ] fan speeds
[ ] fan labels

## License
MIT
