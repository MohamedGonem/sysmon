pub fn get_mem_usage() -> Option<(super::types::MemStat, super::types::MemStat)> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;

    let mut mem_available = 0u64;
    let mut mem_total = 0u64;
    let mut swap_available = 0u64;
    let mut swap_total = 0u64;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("MemTotal:") => mem_total = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0),
            Some("MemAvailable:") => {
                mem_available = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0)
            }
            Some("SwapTotal:") => {
                swap_total = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0)
            }
            Some("SwapFree:") => {
                swap_available = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0)
            }
            _ => {}
        }
    }

    if mem_total == 0 {
        return None;
    }

    let mem_used = mem_total - mem_available;
    let mem_percentage: f64 = (mem_used * 100) as f64 / mem_total as f64;

    let mem_free_mb = mem_available / 1024;
    let mem_used_mb = mem_used / 1024;
    let mem_total_mb = mem_total / 1024;

    let swap_state = swap_total != 0;

    let swap_used = if swap_state {
        swap_total - swap_available
    } else {
        0
    };
    let swap_percentage: f64 = if swap_state {
        (swap_used * 100) as f64 / swap_total as f64
    } else {
        0.0
    };

    let swap_free_mb = if swap_state { swap_available / 1024 } else { 0 };
    let swap_used_mb = if swap_state { swap_used / 1024 } else { 0 };
    let swap_total_mb = if swap_state { swap_total / 1024 } else { 0 };

    Some((
        super::types::MemStat {
            total_mb: mem_total_mb,
            used_mb: mem_used_mb,
            percentage: mem_percentage,
            free_mb: mem_free_mb,
            is_active: None,
        },
        super::types::MemStat {
            total_mb: swap_total_mb,
            used_mb: swap_used_mb,
            percentage: swap_percentage,
            free_mb: swap_free_mb,
            is_active: Some(swap_state),
        },
    ))
}
