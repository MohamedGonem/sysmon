/// Parse the first whitespace-separated token from /proc/uptime into seconds.
pub fn parse_uptime_secs(content: &str) -> u64 {
    content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64
}

/// Format a raw uptime in seconds to Xd Xh Xm Xs.
pub fn format_uptime(uptime_seconds: u64) -> String {
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let seconds = uptime_seconds % 60;
    format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
}

/// Extract PRETTY_NAME from the contents of /etc/os-release.
pub fn parse_distro(content: &str) -> Option<String> {
    content
        .lines()
        .find(|line| line.starts_with("PRETTY_NAME="))
        .and_then(|line| {
            line.split('=')
                .nth(1)
                .map(|name| name.trim_matches('"').to_string())
        })
}

/// Extract btime from the contents of /proc/stat.
pub fn parse_boot_time(content: &str) -> Option<u64> {
    for line in content.lines() {
        if line.starts_with("btime") {
            return line.split_whitespace().nth(1)?.parse::<u64>().ok();
        }
    }
    None
}
