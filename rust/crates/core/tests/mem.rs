use sysmon_core::types::{MemStat, MemoryStats};

// helpers

fn make_mem(total_mb: u64, used_mb: u64, free_mb: u64, percentage: f64) -> MemStat {
    MemStat {
        total_mb,
        used_mb,
        free_mb,
        available_mb: free_mb,
        buffers_mb: 0,
        cached_mb: 0,
        percentage,
    }
}

// MemStat

#[test]
fn mem_stat_stores_fields() {
    let mem = make_mem(8192, 4096, 4096, 50.0);
    assert_eq!(mem.total_mb, 8192);
    assert_eq!(mem.used_mb, 4096);
    assert_eq!(mem.free_mb, 4096);
    assert!((mem.percentage - 50.0).abs() < f64::EPSILON);
}

// MemoryStats::parse: basic case with swap

#[test]
fn parses_typical_meminfo_with_swap() {
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
    let swap = stats.swap.expect("swap should be present");

    assert_eq!(mem.total_mb, 8192);
    assert_eq!(mem.free_mb, 1024);
    assert_eq!(mem.available_mb, 2048);
    assert_eq!(mem.buffers_mb, 128);
    assert_eq!(mem.cached_mb, 3072 + 256 - 64);
    assert_eq!(mem.used_mb, 6144);

    let expected_percentage = 75.0;
    assert!((mem.percentage - expected_percentage).abs() < 0.01);

    assert_eq!(swap.total_mb, 4096);
    assert_eq!(swap.free_mb, 3072);
    assert_eq!(swap.available_mb, 3072);
    assert_eq!(swap.used_mb, 1024);
    assert_eq!(swap.buffers_mb, 0);
    assert_eq!(swap.cached_mb, 0);
}

// MemoryStats::parse: no swap configured

#[test]
fn parses_meminfo_without_swap() {
    let content = "\
MemTotal:       4194304 kB
MemFree:        1048576 kB
MemAvailable:   2097152 kB
SwapTotal:      0 kB
SwapFree:       0 kB
";
    let stats = MemoryStats::parse(content).expect("should parse");

    assert_eq!(stats.memory.total_mb, 4096);
    assert_eq!(stats.memory.free_mb, 1024);
    assert_eq!(stats.memory.available_mb, 2048);
    assert_eq!(stats.memory.used_mb, 2048);
    assert!(stats.swap.is_none());
}

// MemoryStats::parse: missing MemTotal -> None

#[test]
fn returns_none_when_mem_total_missing() {
    let content = "\
MemAvailable:   2097152 kB
SwapTotal:      0 kB
";
    assert!(MemoryStats::parse(content).is_none());
}

// MemoryStats::parse: MemTotal is zero -> None

#[test]
fn returns_none_when_mem_total_is_zero() {
    let content = "\
MemTotal:       0 kB
MemAvailable:   0 kB
";
    assert!(MemoryStats::parse(content).is_none());
}

// MemoryStats::parse: completely empty content -> None

#[test]
fn returns_none_on_empty_content() {
    assert!(MemoryStats::parse("").is_none());
}

// MemoryStats::parse: unrelated lines are ignored

#[test]
fn ignores_unrecognised_lines() {
    let content = "\
SomeRandomKey:  99999 kB
MemTotal:       2097152 kB
AnotherKey:     12345 kB
MemAvailable:   1048576 kB
";
    let stats = MemoryStats::parse(content).expect("should parse");
    assert_eq!(stats.memory.total_mb, 2048);
    assert_eq!(stats.memory.available_mb, 1024);
    assert_eq!(stats.memory.used_mb, 1024);
}

// MemoryStats::parse: SwapFree absent means all swap is used

#[test]
fn parses_when_swap_free_is_missing_defaults_to_zero() {
    let content = "\
MemTotal:       2097152 kB
MemAvailable:   1048576 kB
SwapTotal:      1048576 kB
";
    let stats = MemoryStats::parse(content).expect("should parse");
    let swap = stats.swap.expect("swap should be present");

    assert_eq!(swap.used_mb, 1024);
    assert_eq!(swap.free_mb, 0);
    assert_eq!(swap.percentage, 100.0);
}

// percentage correctness

#[test]
fn mem_percentage_is_100_when_nothing_available() {
    let content = "\
MemTotal:       1048576 kB
MemFree:             0 kB
MemAvailable:        0 kB
";
    let stats = MemoryStats::parse(content).expect("should parse");
    assert!((stats.memory.percentage - 100.0).abs() < 0.01);
}

#[test]
fn mem_percentage_is_0_when_fully_available() {
    let content = "\
MemTotal:       1048576 kB
MemFree:        1048576 kB
MemAvailable:   1048576 kB
";
    let stats = MemoryStats::parse(content).expect("should parse");
    assert!((stats.memory.percentage - 0.0).abs() < 0.01);
    assert_eq!(stats.memory.used_mb, 0);
}

#[test]
fn mem_percentage_is_50_when_half_used() {
    let content = "\
MemTotal:       2048 kB
MemFree:         1024 kB
MemAvailable:    1024 kB
";
    let stats = MemoryStats::parse(content).expect("should parse");
    assert!((stats.memory.percentage - 50.0).abs() < 0.01);
}

#[test]
fn falls_back_to_free_plus_buffers_plus_cached_when_available_is_missing() {
    let content = "\
MemTotal:       2097152 kB
MemFree:         524288 kB
Buffers:          65536 kB
Cached:          524288 kB
SReclaimable:     65536 kB
Shmem:            32768 kB
";
    let stats = MemoryStats::parse(content).expect("should parse");

    assert_eq!(stats.memory.available_mb, 512 + 64 + 512 + 64 - 32);
    assert_eq!(stats.memory.free_mb, 512);
    assert_eq!(stats.memory.used_mb, 2048 - (512 + 64 + 512 + 64 - 32));
}
