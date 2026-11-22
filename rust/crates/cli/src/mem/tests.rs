use super::types::MemStat;

// helpers

fn make_mem(total_mb: u64, used_mb: u64, free_mb: u64, percentage: f64) -> MemStat {
    MemStat {
        total_mb,
        used_mb,
        free_mb,
        percentage,
        is_active: None,
    }
}

fn make_swap(total_mb: u64, used_mb: u64, free_mb: u64, percentage: f64, active: bool) -> MemStat {
    MemStat {
        total_mb,
        used_mb,
        free_mb,
        percentage,
        is_active: Some(active),
    }
}

// parse_meminfo — mirrors the logic in io::get_mem_usage so we can test it
// without hitting /proc/meminfo on disk.

fn parse_meminfo(content: &str) -> Option<(MemStat, MemStat)> {
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
    let mem_percentage = (mem_used * 100) as f64 / mem_total as f64;
    let mem_free_mb = mem_available / 1024;
    let mem_used_mb = mem_used / 1024;
    let mem_total_mb = mem_total / 1024;

    let swap_state = swap_total != 0;
    let swap_used = if swap_state {
        swap_total - swap_available
    } else {
        0
    };
    let swap_percentage = if swap_state {
        (swap_used * 100) as f64 / swap_total as f64
    } else {
        0.0
    };
    let swap_free_mb = if swap_state { swap_available / 1024 } else { 0 };
    let swap_used_mb = if swap_state { swap_used / 1024 } else { 0 };
    let swap_total_mb = if swap_state { swap_total / 1024 } else { 0 };

    Some((
        MemStat {
            total_mb: mem_total_mb,
            used_mb: mem_used_mb,
            free_mb: mem_free_mb,
            percentage: mem_percentage,
            is_active: None,
        },
        MemStat {
            total_mb: swap_total_mb,
            used_mb: swap_used_mb,
            free_mb: swap_free_mb,
            percentage: swap_percentage,
            is_active: Some(swap_state),
        },
    ))
}

// MemStat type

#[test]
fn mem_stat_stores_fields() {
    let m = make_mem(8192, 4096, 4096, 50.0);
    assert_eq!(m.total_mb, 8192);
    assert_eq!(m.used_mb, 4096);
    assert_eq!(m.free_mb, 4096);
    assert!((m.percentage - 50.0).abs() < f64::EPSILON);
    assert_eq!(m.is_active, None);
}

#[test]
fn swap_stat_is_active_some_true() {
    let s = make_swap(4096, 512, 3584, 12.5, true);
    assert_eq!(s.is_active, Some(true));
}

#[test]
fn swap_stat_is_active_some_false_when_no_swap() {
    let s = make_swap(0, 0, 0, 0.0, false);
    assert_eq!(s.is_active, Some(false));
}

// parse_meminfo: basic case with swap

#[test]
fn parses_typical_meminfo_with_swap() {
    let content = "\
MemTotal:       8388608 kB
MemFree:        1048576 kB
MemAvailable:   2097152 kB
SwapTotal:      4194304 kB
SwapFree:       3145728 kB
";
    let (mem, swap) = parse_meminfo(content).expect("should parse");

    assert_eq!(mem.total_mb, 8192); // 8388608 / 1024
    assert_eq!(mem.free_mb, 2048); // 2097152 / 1024
    assert_eq!(mem.used_mb, 6144); // (8388608 - 2097152) / 1024
    let expected_pct = (6144.0 * 1024.0 * 100.0) / (8192.0 * 1024.0);
    assert!((mem.percentage - expected_pct).abs() < 0.01);

    assert_eq!(swap.total_mb, 4096);
    assert_eq!(swap.free_mb, 3072);
    assert_eq!(swap.used_mb, 1024);
    assert_eq!(swap.is_active, Some(true));
}

// parse_meminfo: no swap configured

#[test]
fn parses_meminfo_without_swap() {
    let content = "\
MemTotal:       4194304 kB
MemAvailable:   2097152 kB
SwapTotal:      0 kB
SwapFree:       0 kB
";
    let (mem, swap) = parse_meminfo(content).expect("should parse");

    assert_eq!(mem.total_mb, 4096);
    assert_eq!(mem.used_mb, 2048);
    assert_eq!(swap.total_mb, 0);
    assert_eq!(swap.used_mb, 0);
    assert_eq!(swap.percentage, 0.0);
    assert_eq!(swap.is_active, Some(false));
}

// parse_meminfo: missing MemTotal → None

#[test]
fn returns_none_when_mem_total_missing() {
    let content = "\
MemAvailable:   2097152 kB
SwapTotal:      0 kB
";
    assert!(parse_meminfo(content).is_none());
}

// parse_meminfo: MemTotal is zero → None

#[test]
fn returns_none_when_mem_total_is_zero() {
    let content = "\
MemTotal:       0 kB
MemAvailable:   0 kB
";
    assert!(parse_meminfo(content).is_none());
}

// parse_meminfo: completely empty content → None

#[test]
fn returns_none_on_empty_content() {
    assert!(parse_meminfo("").is_none());
}

// parse_meminfo: unrelated lines are ignored

#[test]
fn ignores_unrecognised_lines() {
    let content = "\
SomeRandomKey:  99999 kB
MemTotal:       2097152 kB
AnotherKey:     12345 kB
MemAvailable:   1048576 kB
";
    let (mem, _) = parse_meminfo(content).expect("should parse");
    assert_eq!(mem.total_mb, 2048);
    assert_eq!(mem.used_mb, 1024);
}

// parse_meminfo: fields without a unit suffix still parse (parse() handles digits-only)

#[test]
fn parses_when_swap_free_is_missing_defaults_to_zero() {
    // SwapFree absent → swap_available stays 0 → all swap is "used"
    let content = "\
MemTotal:       2097152 kB
MemAvailable:   1048576 kB
SwapTotal:      1048576 kB
";
    let (_, swap) = parse_meminfo(content).expect("should parse");
    assert_eq!(swap.used_mb, 1024); // entire swap is used
    assert_eq!(swap.free_mb, 0);
    assert_eq!(swap.percentage, 100.0);
    assert_eq!(swap.is_active, Some(true));
}

// percentage correctness

#[test]
fn mem_percentage_is_100_when_nothing_available() {
    let content = "\
MemTotal:       1048576 kB
MemAvailable:   0 kB
";
    let (mem, _) = parse_meminfo(content).expect("should parse");
    assert!((mem.percentage - 100.0).abs() < 0.01);
}

#[test]
fn mem_percentage_is_0_when_fully_available() {
    let content = "\
MemTotal:       1048576 kB
MemAvailable:   1048576 kB
";
    let (mem, _) = parse_meminfo(content).expect("should parse");
    assert!((mem.percentage - 0.0).abs() < 0.01);
    assert_eq!(mem.used_mb, 0);
}

#[test]
fn mem_percentage_is_50_when_half_used() {
    let content = "\
MemTotal:       2048 kB
MemAvailable:   1024 kB
";
    let (mem, _) = parse_meminfo(content).expect("should parse");
    assert!((mem.percentage - 50.0).abs() < 0.01);
}
