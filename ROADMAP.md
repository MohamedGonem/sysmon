# Roadmap

Lots of things need to be reworked.

Linux core monitoring is implemented. The unchecked general goals are the
cross-platform, GUI, and daemon work that remains.

## General Goals

- [x] drop `clap` and manually parse args
- [ ] add support for `Windows` and `macOS` (CLI)
- [ ] add support for `Android` and `IOS` via `kotlin` and `compose` (GUI)
- [ ] add `iced` (GUI) for desktop versions
- [ ] add daemon mode to run with other tools

## Core Features (Linux)

CPU
---
- [x] model name
- [x] vendor id
- [x] cache type, level and size
- [x] flags
- [x] physical cores count
- [x] logical cores count
- [x] current frequency
- [x] min frequency
- [x] max frequency
- [x] governor
- [x] temperature
- [x] usage percentage
- [x] load avg 1min
- [x] load avg 5min
- [x] load avg 15min
- [x] context switches
- [x] procs running
- [x] per core:
   - [x] id (stable `cpuN` id)
   - [x] usage percentage
   - [x] temperature
   - [x] current frequency
   - [x] governor
   - [x] online bool
   - [x] user
   - [x] nice
   - [x] system
   - [x] idle
   - [x] iowait
   - [x] irq
   - [x] softirq
   - [x] steal
   - [x] guest
   - [x] guest_nice

MEMORY & SWAP
-------------
- [x] total ram
- [x] used ram
- [x] free ram (`MemFree`)
- [x] available ram (`MemAvailable`, with `MemFree + Buffers + Cached` fallback)
- [x] buffers
- [x] cached (`Cached + SReclaimable - Shmem`)
- [x] total swap
- [x] used swap
- [x] free swap

SYSTEM INFO
-----------
- [x] hostname
- [x] kernel version
- [x] uptime (raw seconds + formatted d/h/m/s)
- [x] distro name (/etc/os-release)
- [x] architecture
- [x] boot time (formatted local date/time)

BATTERY
-------
- [x] status (Charging/Discharging/Full)
- [x] capacity percent (falls back to energy/charge ratio)
- [x] energy now (or charge now)
- [x] energy full (or charge full)
- [x] energy full design (or charge full design)
- [x] power draw (watts, from `power_now` or `current_now * voltage_now`)
- [x] voltage
- [x] cycle count
- [x] health percent (energy_full / energy_full_design * 100)

NETWORK
-------
- [x] per interface:
   - [x] name
   - [x] bytes received total
   - [x] bytes sent total
   - [x] download speed (delta)
   - [x] upload speed (delta)
   - [x] packets received
   - [x] packets sent
   - [x] errors in
   - [x] errors out
   - [x] drop in
   - [x] drop out
   - [x] is up (IFF_UP/operstate)
   - [x] mac address
   - [x] ip address (IPv4 and IPv6)

DISK
----
- [x] per mount:
   - [x] device
   - [x] mount point
   - [x] filesystem type
   - [x] total space
   - [x] used space
   - [x] free space
   - [x] available space
   - [x] usage percent
- [x] per device:
   - [x] name
   - [x] reads completed
   - [x] writes completed
   - [x] bytes read
   - [x] bytes written
   - [x] read speed (delta)
   - [x] write speed (delta)
   - [x] is rotational (HDD vs SSD)

PROCESSES
---------
- [x] per process:
   - [x] pid
   - [x] name
   - [x] state
   - [x] cpu usage percent (delta)
   - [x] memory rss
   - [x] memory percent
   - [x] virtual memory size
   - [x] user/owner
   - [x] threads count
   - [x] open file descriptors count (permission-limited fields show `N/A`)
   - [x] command line
   - [x] parent pid
   - [x] nice value
   - [x] start time
- [x] search by text or regex (PID, name, user, and command line)

TEMPERATURES & FANS
-------------------
- [x] cpu temperature
- [x] per core temperature
- [x] disk temperatures
- [x] fan speeds
- [x] fan labels

## CLI

- [x] show system, CPU, cores, memory/swap, and battery
- [x] show network, disk, processes, and sensors
- [x] process search with literal text or regex (`-s`, `-r`)
- [x] detailed output (`-d`)
- [x] tmux output (`-e tmux`)
- [x] manual argument parsing, short-flag clusters, and `--help`/`--version`
