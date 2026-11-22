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

    /// Human-readable boot time in the local timezone.
    pub fn boot_time_display(&self) -> Option<String> {
        self.boot_time.map(format_epoch)
    }
}

/// Format a Unix timestamp in the local timezone as `YYYY-MM-DD HH:MM:SS`.
pub fn format_epoch(epoch: u64) -> String {
    let timestamp = epoch as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();

    // SAFETY: `localtime_r` writes into the provided `tm` and the pointer is valid
    // for the duration of the call. It does not retain the pointer.
    let result = unsafe { libc::localtime_r(&timestamp, local.as_mut_ptr()) };
    if result.is_null() {
        return epoch.to_string();
    }

    // SAFETY: a non-null return from `localtime_r` means `local` was initialised.
    let local = unsafe { local.assume_init() };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        local.tm_year + 1900,
        local.tm_mon + 1,
        local.tm_mday,
        local.tm_hour,
        local.tm_min,
        local.tm_sec
    )
}
