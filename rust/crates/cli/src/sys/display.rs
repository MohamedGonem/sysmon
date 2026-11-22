use super::types::SysStat;
use crate::utils::color_head;
pub fn display_sys(stat: SysStat, tmux: bool) {
    println!("Hostname:    {}", color_head(&stat.hostname, tmux));
    println!("Distro:      {}", color_head(&stat.distro, tmux));
    println!("Kernel:      {}", color_head(&stat.kernel, tmux));
    println!("Arch:        {}", color_head(&stat.arch, tmux));
    println!(
        "Uptime:      {}\n\t\t{}",
        color_head(&stat.uptime_fmt, tmux),
        color_head(&stat.uptime_secs.to_string(), tmux)
    );
    if let Some(bt) = stat.boot_time {
        println!("Boot time:   {}", color_head(&bt.to_string(), tmux));
    }
}
