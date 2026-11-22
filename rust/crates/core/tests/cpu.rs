use sysmon_core::cpu::CpuTimes;
use sysmon_core::types::UsageStat;

// helpers

fn stat(total: u64, idle: u64) -> UsageStat {
    UsageStat {
        user: total.saturating_sub(idle),
        idle,
        ..Default::default()
    }
}

// UsageStat::from_proc_stat_line

#[test]
fn parses_cpu_line_all_fields() {
    let stat = UsageStat::from_proc_stat_line("cpu  1 2 3 4 5 6 7 8 9 10");
    assert_eq!(stat.user, 1);
    assert_eq!(stat.nice, 2);
    assert_eq!(stat.system, 3);
    assert_eq!(stat.idle, 4);
    assert_eq!(stat.iowait, 5);
    assert_eq!(stat.irq, 6);
    assert_eq!(stat.softirq, 7);
    assert_eq!(stat.steal, 8);
    assert_eq!(stat.guest, 9);
    assert_eq!(stat.guest_nice, 10);
}

#[test]
fn parses_core_line() {
    let stat = UsageStat::from_proc_stat_line("cpu0 10 20 30 40 50 60 70 80 90 100");
    assert_eq!(stat.user, 10);
    assert_eq!(stat.idle, 40);
    assert_eq!(stat.iowait, 50);
}

#[test]
fn parses_line_with_extra_whitespace() {
    let stat = UsageStat::from_proc_stat_line("cpu   100 0 50 800 10 0 0 0 0 0");
    assert_eq!(stat.user, 100);
    assert_eq!(stat.system, 50);
    assert_eq!(stat.idle, 800);
}

#[test]
fn parses_empty_line_as_zeros() {
    let stat = UsageStat::from_proc_stat_line("cpu");
    assert_eq!(stat, UsageStat::default());
}

#[test]
fn parses_partial_line_remaining_fields_are_zero() {
    let stat = UsageStat::from_proc_stat_line("cpu 100 200 300");
    assert_eq!(stat.user, 100);
    assert_eq!(stat.nice, 200);
    assert_eq!(stat.system, 300);
    assert_eq!(stat.idle, 0);
    assert_eq!(stat.steal, 0);
}

#[test]
fn parses_non_numeric_field_as_zero() {
    let stat = UsageStat::from_proc_stat_line("cpu 100 bad 300 400 500 600 700 800 900 1000");
    assert_eq!(stat.user, 100);
    assert_eq!(stat.nice, 0);
    assert_eq!(stat.system, 300);
}

// UsageStat::total / idle_total

#[test]
fn total_sums_all_fields() {
    let stat = UsageStat {
        user: 10,
        nice: 20,
        system: 30,
        idle: 40,
        iowait: 50,
        irq: 60,
        softirq: 70,
        steal: 80,
        guest: 90,
        guest_nice: 100,
    };

    assert_eq!(stat.total(), 550);
    assert_eq!(stat.idle_total(), 90);
}

#[test]
fn total_all_zeros() {
    let stat = UsageStat::default();
    assert_eq!(stat.total(), 0);
    assert_eq!(stat.idle_total(), 0);
}

#[test]
fn total_only_idle() {
    let stat = stat(100, 100);
    assert_eq!(stat.total(), 100);
    assert_eq!(stat.idle_total(), 100);
}

#[test]
fn total_only_iowait_counts_as_idle() {
    let stat = UsageStat {
        iowait: 50,
        ..Default::default()
    };
    assert_eq!(stat.total(), 50);
    assert_eq!(stat.idle_total(), 50);
}

// UsageStat::usage_since

#[test]
fn usage_is_100_percent_when_nothing_was_idle() {
    assert_eq!(stat(100, 0).usage_since(&stat(0, 0)), 100.0);
}

#[test]
fn usage_is_0_percent_when_everything_was_idle() {
    assert_eq!(stat(100, 100).usage_since(&stat(0, 0)), 0.0);
}

#[test]
fn usage_is_50_percent_when_half_was_idle() {
    assert_eq!(stat(100, 50).usage_since(&stat(0, 0)), 50.0);
}

#[test]
fn usage_uses_only_the_delta() {
    // total delta = 100, idle delta = 20 -> 80%
    assert_eq!(stat(200, 60).usage_since(&stat(100, 40)), 80.0);
}

#[test]
fn usage_with_large_real_world_values() {
    let earlier = stat(1_000_000, 800_000);
    let later = stat(1_001_000, 800_500);
    assert_eq!(later.usage_since(&earlier), 50.0);
}

#[test]
fn usage_is_zero_when_no_time_passed() {
    let sample = stat(100, 50);
    assert_eq!(sample.usage_since(&sample), 0.0);
}

#[test]
fn full_pipeline_produces_expected_usage() {
    let line1 = "cpu 1000 0 500 8000 500 0 0 0 0 0";
    let line2 = "cpu 1100 0 600 9000 500 0 0 0 0 0";

    let earlier = UsageStat::from_proc_stat_line(line1);
    let later = UsageStat::from_proc_stat_line(line2);

    // total delta = 1200, idle delta = 1000 -> 1 - 1000/1200 ~= 16.67%
    let expected = 100.0 * (1.0 - 1000.0_f64 / 1200.0_f64);
    assert!((later.usage_since(&earlier) - expected).abs() < 0.001);
}

// CpuTimes

#[test]
fn parses_aggregate_and_core_times() {
    let content = "\
cpu  100 0 50 800 10 0 0 0 0 0
cpu0 50 0 25 400 5 0 0 0 0 0
cpu1 50 0 25 400 5 0 0 0 0 0
intr 12345
";
    let times = CpuTimes::parse(content).expect("should parse /proc/stat");

    assert_eq!(times.total.user, 100);
    assert_eq!(times.total.idle, 800);
    assert_eq!(times.cores.len(), 2);
    assert_eq!(times.cores[0].user, 50);
    assert_eq!(times.cores[1].user, 50);
}

#[test]
fn returns_none_without_aggregate_line() {
    assert!(CpuTimes::parse("cpu0 1 2 3 4\n").is_none());
}

#[test]
fn computes_aggregate_and_per_core_usage() {
    let earlier = CpuTimes::parse(
        "\
cpu  100 0 50 800 10 0 0 0 0 0
cpu0 50 0 25 400 5 0 0 0 0 0
cpu1 50 0 25 400 5 0 0 0 0 0
",
    )
    .unwrap();
    let later = CpuTimes::parse(
        "\
cpu  110 0 55 880 10 0 0 0 0 0
cpu0 55 0 27 440 5 0 0 0 0 0
cpu1 55 0 28 440 5 0 0 0 0 0
",
    )
    .unwrap();

    assert!(later.usage_since(&earlier) > 0.0);

    let core_usages = later.core_usages_since(&earlier);
    assert_eq!(core_usages.len(), 2);
    assert!(
        core_usages
            .iter()
            .all(|usage| (0.0..=100.0).contains(usage))
    );
}
