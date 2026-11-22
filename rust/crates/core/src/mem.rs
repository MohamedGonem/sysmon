use crate::types::{MemStat, MemoryStats};

impl MemoryStats {
    /// Read and parse `/proc/meminfo`.
    pub fn read() -> Option<Self> {
        let content = std::fs::read_to_string("/proc/meminfo").ok()?;
        Self::parse(&content)
    }

    /// Parse the contents of `/proc/meminfo`.
    pub fn parse(content: &str) -> Option<Self> {
        let mut mem_total = 0u64;
        let mut mem_available = 0u64;
        let mut swap_total = 0u64;
        let mut swap_free = 0u64;

        for line in content.lines() {
            let mut parts = line.split_whitespace();
            let Some(key) = parts.next() else {
                continue;
            };
            let Some(value) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
                continue;
            };

            match key {
                "MemTotal:" => mem_total = value,
                "MemAvailable:" => mem_available = value,
                "SwapTotal:" => swap_total = value,
                "SwapFree:" => swap_free = value,
                _ => {}
            }
        }

        if mem_total == 0 {
            return None;
        }

        Some(Self {
            memory: MemStat::from_kb(mem_total, mem_available),
            swap: (swap_total != 0).then(|| MemStat::from_kb(swap_total, swap_free)),
        })
    }
}

impl MemStat {
    fn from_kb(total_kb: u64, available_kb: u64) -> Self {
        let used_kb = total_kb.saturating_sub(available_kb);

        Self {
            total_mb: total_kb / 1024,
            used_mb: used_kb / 1024,
            free_mb: available_kb / 1024,
            percentage: if total_kb == 0 {
                0.0
            } else {
                used_kb as f64 * 100.0 / total_kb as f64
            },
        }
    }
}
