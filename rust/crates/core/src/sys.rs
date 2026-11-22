use crate::types::SysStat;

impl SysStat {
    /// Read system information from `/proc` and `/etc/os-release`.
    pub fn read() -> Self {
        let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        let distro = std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| Self::parse_distro(&content))
            .unwrap_or_else(|| "Unknown".to_string());

        let uptime_secs = std::fs::read_to_string("/proc/uptime")
            .ok()
            .map(|content| Self::parse_uptime_secs(&content))
            .unwrap_or(0);

        let boot_time = std::fs::read_to_string("/proc/stat")
            .ok()
            .and_then(|content| Self::parse_boot_time(&content));

        Self {
            hostname,
            kernel,
            distro,
            arch: std::env::consts::ARCH.to_string(),
            uptime_secs,
            boot_time,
        }
    }

    /// Parse the first whitespace-separated token from `/proc/uptime` into seconds.
    pub fn parse_uptime_secs(content: &str) -> u64 {
        content
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0) as u64
    }

    /// Extract `PRETTY_NAME` from the contents of `/etc/os-release`.
    pub fn parse_distro(content: &str) -> Option<String> {
        content
            .lines()
            .find(|line| line.starts_with("PRETTY_NAME="))
            .and_then(|line| {
                line.split_once('=')
                    .map(|(_, name)| name.trim_matches(['"', '\'']).to_string())
            })
    }

    /// Extract `btime` from the contents of `/proc/stat`.
    pub fn parse_boot_time(content: &str) -> Option<u64> {
        content
            .lines()
            .find(|line| line.starts_with("btime"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<u64>().ok())
    }
}
