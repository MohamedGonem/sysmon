pub fn get_hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string()
}

pub fn get_kernel_version() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string()
}

// uptime (raw seconds + formatted d/h/m/s)
pub fn get_uptime() -> (u64, String) {
    let content = std::fs::read_to_string("/proc/uptime").unwrap_or_else(|_| "0 0".to_string());
    let uptime_seconds = content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;

    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let seconds = uptime_seconds % 60;

    let formatted_uptime = format!("{}d {}h {}m {}s", days, hours, minutes, seconds);

    (uptime_seconds, formatted_uptime)
}

pub fn get_distribution() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("PRETTY_NAME="))
                .and_then(|line| {
                    line.split('=')
                        .nth(1)
                        .map(|name| name.trim_matches('"').to_string())
                })
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

pub fn get_architecture() -> String {
    std::env::consts::ARCH.to_string()
}

pub fn get_boot_time() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    for line in content.lines() {
        if line.starts_with("btime") {
            return line.split_whitespace().nth(1)?.parse::<u64>().ok();
        }
    }
    None
}
