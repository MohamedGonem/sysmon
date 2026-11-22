use crate::types::{MemStat, MemoryStats};

impl MemoryStats {
    /// Read and parse `/proc/meminfo`.
    pub fn read() -> Option<Self> {
        let content = std::fs::read_to_string("/proc/meminfo").ok()?;
        Self::parse(&content)
    }

    /// Parse the contents of `/proc/meminfo`.
    ///
    /// `MemAvailable` is used when the kernel provides it. Older kernels only expose
    /// `MemFree`, so the usual `MemFree + Buffers + Cached` estimate is used as a fallback.
    pub fn parse(content: &str) -> Option<Self> {
        let mut mem_total = 0u64;
        let mut mem_free = 0u64;
        let mut mem_available = None;
        let mut buffers = 0u64;
        let mut cached = 0u64;
        let mut sreclaimable = 0u64;
        let mut shmem = 0u64;
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
                "MemFree:" => mem_free = value,
                "MemAvailable:" => mem_available = Some(value),
                "Buffers:" => buffers = value,
                "Cached:" => cached = value,
                "SReclaimable:" => sreclaimable = value,
                "Shmem:" => shmem = value,
                "SwapTotal:" => swap_total = value,
                "SwapFree:" => swap_free = value,
                _ => {}
            }
        }

        if mem_total == 0 {
            return None;
        }

        // The kernel's "Cached" figure excludes reclaimable slab; add it back and
        // remove shared memory, which is already counted as used by most tools.
        let cached = cached.saturating_add(sreclaimable).saturating_sub(shmem);
        let available = mem_available
            .unwrap_or_else(|| mem_free.saturating_add(buffers).saturating_add(cached));

        Some(Self {
            memory: MemStat::from_kb(mem_total, mem_free, available, buffers, cached),
            swap: (swap_total != 0).then(|| MemStat::swap_from_kb(swap_total, swap_free)),
        })
    }
}

impl MemStat {
    fn from_kb(
        total_kb: u64,
        free_kb: u64,
        available_kb: u64,
        buffers_kb: u64,
        cached_kb: u64,
    ) -> Self {
        let available_kb = available_kb.min(total_kb);
        let used_kb = total_kb.saturating_sub(available_kb);

        Self {
            total_mb: total_kb / 1024,
            used_mb: used_kb / 1024,
            free_mb: free_kb.min(total_kb) / 1024,
            available_mb: available_kb / 1024,
            buffers_mb: buffers_kb.min(total_kb) / 1024,
            cached_mb: cached_kb.min(total_kb) / 1024,
            percentage: if total_kb == 0 {
                0.0
            } else {
                used_kb as f64 * 100.0 / total_kb as f64
            },
        }
    }

    fn swap_from_kb(total_kb: u64, free_kb: u64) -> Self {
        let free_kb = free_kb.min(total_kb);
        let used_kb = total_kb.saturating_sub(free_kb);

        Self {
            total_mb: total_kb / 1024,
            used_mb: used_kb / 1024,
            free_mb: free_kb / 1024,
            available_mb: free_kb / 1024,
            buffers_mb: 0,
            cached_mb: 0,
            percentage: if total_kb == 0 {
                0.0
            } else {
                used_kb as f64 * 100.0 / total_kb as f64
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_mem_free_and_mem_available_separately() {
        let content = "\
MemTotal:       8388608 kB
MemFree:        1048576 kB
MemAvailable:   2097152 kB
Buffers:         131072 kB
Cached:         3145728 kB
SReclaimable:    262144 kB
Shmem:            65536 kB
SwapTotal:      4194304 kB
SwapFree:       3145728 kB
";
        let stats = MemoryStats::parse(content).expect("should parse");
        let mem = stats.memory;

        assert_eq!(mem.total_mb, 8192);
        assert_eq!(mem.free_mb, 1024);
        assert_eq!(mem.available_mb, 2048);
        assert_eq!(mem.buffers_mb, 128);
        assert_eq!(mem.cached_mb, 3072 + 256 - 64);
        assert_eq!(mem.used_mb, 6144);
        assert!((mem.percentage - 75.0).abs() < 0.01);
    }

    #[test]
    fn falls_back_to_free_plus_buffers_plus_cached() {
        let content = "\
MemTotal:       2097152 kB
MemFree:         524288 kB
Buffers:          65536 kB
Cached:          524288 kB
SReclaimable:     65536 kB
Shmem:            32768 kB
";
        let stats = MemoryStats::parse(content).expect("should parse");
        let mem = stats.memory;

        assert_eq!(mem.available_mb, 512 + 64 + 512 + 64 - 32);
        assert_eq!(mem.free_mb, 512);
        assert_eq!(mem.used_mb, 2048 - (512 + 64 + 512 + 64 - 32));
    }

    #[test]
    fn swap_uses_free_as_available() {
        let content = "\
MemTotal:       1048576 kB
MemFree:         524288 kB
SwapTotal:      1048576 kB
SwapFree:        262144 kB
";
        let stats = MemoryStats::parse(content).expect("should parse");
        let swap = stats.swap.expect("swap should be present");

        assert_eq!(swap.total_mb, 1024);
        assert_eq!(swap.used_mb, 768);
        assert_eq!(swap.free_mb, 256);
        assert_eq!(swap.available_mb, 256);
        assert_eq!(swap.buffers_mb, 0);
        assert_eq!(swap.cached_mb, 0);
        assert!((swap.percentage - 75.0).abs() < 0.01);
    }
}
