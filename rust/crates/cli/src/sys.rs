use crate::utils::color_head;
use sysmon_core::types::SysStat;

pub fn print_sys(tmux: bool) {
    display_sys(SysStat::read(), tmux);
}

fn display_sys(stat: SysStat, tmux: bool) {
    println!("Hostname:    {}", color_head(&stat.hostname, tmux));
    println!("Distro:      {}", color_head(&stat.distro, tmux));
    println!("Kernel:      {}", color_head(&stat.kernel, tmux));
    println!("Arch:        {}", color_head(&stat.arch, tmux));
    println!(
        "Uptime:      {}\n\t\t{}",
        color_head(&format_uptime(stat.uptime_secs), tmux),
        color_head(&stat.uptime_secs.to_string(), tmux)
    );
    if let Some(boot_time) = stat.boot_time {
        println!("Boot time:   {}", color_head(&boot_time.to_string(), tmux));
    }
}

/// Format a raw uptime in seconds as `Xd Xh Xm Xs`.
fn format_uptime(uptime_seconds: u64) -> String {
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let seconds = uptime_seconds % 60;
    format!("{days}d {hours}h {minutes}m {seconds}s")
}

#[cfg(test)]
mod tests {
    use super::format_uptime;

    #[test]
    fn formats_zero_uptime() {
        assert_eq!(format_uptime(0), "0d 0h 0m 0s");
    }

    #[test]
    fn formats_mixed_uptime() {
        assert_eq!(format_uptime(93784), "1d 2h 3m 4s");
    }
}
