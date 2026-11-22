use super::parser;
use super::types::SysStat;

pub fn get_sys_stat() -> SysStat {
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
        .and_then(|c| parser::parse_distro(&c))
        .unwrap_or_else(|| "Unknown".to_string());

    let arch = std::env::consts::ARCH.to_string();

    let uptime_content =
        std::fs::read_to_string("/proc/uptime").unwrap_or_else(|_| "0 0".to_string());
    let uptime_secs = parser::parse_uptime_secs(&uptime_content);
    let uptime_fmt = parser::format_uptime(uptime_secs);

    let boot_time = std::fs::read_to_string("/proc/stat")
        .ok()
        .and_then(|c| parser::parse_boot_time(&c));

    SysStat {
        hostname,
        kernel,
        distro,
        arch,
        uptime_secs,
        uptime_fmt,
        boot_time,
    }
}
