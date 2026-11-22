use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::types::{ProcessStat, ProcessStats, SysStat};

impl ProcessStats {
    /// Read a snapshot of every process visible in `/proc`.
    pub fn read() -> Option<Self> {
        Some(Self {
            processes: snapshot()?.processes,
        })
    }

    /// Sample process CPU time over a window.
    ///
    /// A process with multiple busy threads can exceed 100%, matching the
    /// behaviour of tools such as `top`.
    pub fn sample(window: Duration) -> Option<Self> {
        let start = Instant::now();
        let earlier = snapshot()?;
        std::thread::sleep(window);
        let later = snapshot()?;
        let elapsed = start.elapsed().as_secs_f64().max(0.001);
        let clock_ticks = clock_ticks_per_second() as f64;

        let mut processes = later.processes;
        for process in &mut processes {
            let Some(previous_ticks) = earlier.ticks.get(&process.pid) else {
                continue;
            };
            let current_ticks = later
                .ticks
                .get(&process.pid)
                .copied()
                .unwrap_or(*previous_ticks);
            let delta = current_ticks.saturating_sub(*previous_ticks);

            process.cpu_usage_percentage = delta as f64 * 100.0 / clock_ticks / elapsed;
        }

        Some(Self { processes })
    }
}

struct Snapshot {
    processes: Vec<ProcessStat>,
    ticks: HashMap<u32, u64>,
}

fn snapshot() -> Option<Snapshot> {
    let total_memory = total_memory_bytes().unwrap_or(0);
    let boot_time = boot_time();
    let clock_ticks = clock_ticks_per_second();
    let page_size = page_size_bytes();
    let users = users_by_uid();
    let entries = fs::read_dir("/proc").ok()?;
    let mut processes = Vec::new();
    let mut ticks = HashMap::new();

    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };

        let directory = entry.path();
        let Ok(stat_content) = fs::read_to_string(directory.join("stat")) else {
            continue;
        };
        let Some(stat) = parse_stat(&stat_content) else {
            continue;
        };

        let uid = fs::read_to_string(directory.join("status"))
            .ok()
            .and_then(|status| parse_uid(&status));
        let user = uid
            .map(|uid| users.get(&uid).cloned().unwrap_or_else(|| uid.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let rss_bytes = stat.rss_pages.saturating_mul(page_size);
        let memory_percentage = if total_memory == 0 {
            0.0
        } else {
            rss_bytes as f64 * 100.0 / total_memory as f64
        };
        let command_line = read_cmdline(&directory).unwrap_or_else(|| stat.comm.clone());
        let open_fds = fs::read_dir(directory.join("fd"))
            .ok()
            .map(|entries| entries.flatten().count() as u64);

        ticks.insert(pid, stat.utime.saturating_add(stat.stime));
        processes.push(ProcessStat {
            pid,
            name: stat.comm,
            state: stat.state,
            cpu_usage_percentage: 0.0,
            memory_rss_bytes: rss_bytes,
            memory_percentage,
            virtual_memory_bytes: stat.vsize,
            user,
            threads: stat.threads,
            open_fds,
            command_line,
            parent_pid: stat.ppid,
            nice: stat.nice,
            start_time: boot_time.saturating_add(stat.starttime / clock_ticks),
        });
    }

    processes.sort_by_key(|process| process.pid);
    Some(Snapshot { processes, ticks })
}

#[derive(Debug, PartialEq, Eq)]
struct RawStat {
    comm: String,
    state: char,
    ppid: u32,
    utime: u64,
    stime: u64,
    nice: i32,
    threads: u64,
    starttime: u64,
    vsize: u64,
    rss_pages: u64,
}

fn parse_stat(content: &str) -> Option<RawStat> {
    let open = content.find('(')?;
    let close = content.rfind(')')?;
    if close <= open {
        return None;
    }

    let comm = content[open + 1..close].to_string();
    let fields: Vec<_> = content[close + 1..].split_whitespace().collect();
    if fields.len() < 22 {
        return None;
    }

    Some(RawStat {
        comm,
        state: fields[0].chars().next()?,
        ppid: fields[1].parse().ok()?,
        utime: fields[11].parse().ok()?,
        stime: fields[12].parse().ok()?,
        nice: fields[16].parse().ok()?,
        threads: fields[17].parse().ok()?,
        starttime: fields[19].parse().ok()?,
        vsize: fields[20].parse().ok()?,
        rss_pages: fields[21].parse().ok()?,
    })
}

fn parse_uid(status: &str) -> Option<u32> {
    status.lines().find_map(|line| {
        let rest = line.strip_prefix("Uid:")?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

fn parse_passwd(content: &str) -> HashMap<u32, String> {
    content
        .lines()
        .filter_map(|line| {
            let mut fields = line.split(':');
            let name = fields.next()?;
            let _password = fields.next()?;
            let uid = fields.next()?.parse().ok()?;
            Some((uid, name.to_string()))
        })
        .collect()
}

fn users_by_uid() -> HashMap<u32, String> {
    fs::read_to_string("/etc/passwd")
        .ok()
        .map(|content| parse_passwd(&content))
        .unwrap_or_default()
}

fn read_cmdline(directory: &Path) -> Option<String> {
    let bytes = fs::read(directory.join("cmdline")).ok()?;
    let command = bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).into_owned())
        .collect::<Vec<_>>()
        .join(" ");

    (!command.is_empty()).then_some(command)
}

fn total_memory_bytes() -> Option<u64> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;

    content.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        if fields.next()? != "MemTotal:" {
            return None;
        }
        fields.next()?.parse::<u64>().ok().map(|kb| kb * 1024)
    })
}

fn boot_time() -> u64 {
    fs::read_to_string("/proc/stat")
        .ok()
        .and_then(|content| SysStat::parse_boot_time(&content))
        .unwrap_or(0)
}

fn clock_ticks_per_second() -> u64 {
    // SAFETY: `sysconf` has no preconditions and only reads process-global state.
    let value = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if value > 0 { value as u64 } else { 100 }
}

fn page_size_bytes() -> u64 {
    // SAFETY: `sysconf` has no preconditions and only reads process-global state.
    let value = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if value > 0 { value as u64 } else { 4096 }
}

#[cfg(test)]
mod tests {
    use super::{parse_passwd, parse_stat, parse_uid};

    #[test]
    fn parses_stat_with_spaces_and_parentheses_in_comm() {
        let content =
            "1234 (my (weird) name) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21";
        let stat = parse_stat(content).expect("should parse");

        assert_eq!(stat.comm, "my (weird) name");
        assert_eq!(stat.state, 'S');
        assert_eq!(stat.ppid, 1);
        assert_eq!(stat.utime, 11);
        assert_eq!(stat.stime, 12);
        assert_eq!(stat.nice, 16);
        assert_eq!(stat.threads, 17);
        assert_eq!(stat.starttime, 19);
        assert_eq!(stat.vsize, 20);
        assert_eq!(stat.rss_pages, 21);
    }

    #[test]
    fn rejects_truncated_stat_lines() {
        assert!(parse_stat("1234 (short) S 1 2 3").is_none());
        assert!(parse_stat("not a stat line").is_none());
    }

    #[test]
    fn parses_effective_uid() {
        let status = "Name:\ttest\nUid:\t1000\t1001\t1000\t1001\n";
        assert_eq!(parse_uid(status), Some(1000));
        assert_eq!(parse_uid("Name:\ttest\n"), None);
    }

    #[test]
    fn parses_passwd_entries() {
        let users = parse_passwd(
            "root:x:0:0:root:/root:/bin/bash\ncrimson:x:1000:1000::/home/crimson:/bin/zsh\n",
        );

        assert_eq!(users.get(&0).map(String::as_str), Some("root"));
        assert_eq!(users.get(&1000).map(String::as_str), Some("crimson"));
    }
}
