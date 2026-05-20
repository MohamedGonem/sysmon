use super::parser::{get_total_from_stat, merge_totals, parse_stat_line};
use super::types::UsageStat;

// helpers

fn make_stat(
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
    guest: u64,
    guest_nice: u64,
) -> UsageStat {
    UsageStat {
        user,
        nice,
        system,
        idle,
        iowait,
        irq,
        softirq,
        steal,
        guest,
        guest_nice,
    }
}

// parse_stat_line

#[test]
fn parses_cpu_line_all_fields() {
    let stat = parse_stat_line("cpu  1 2 3 4 5 6 7 8 9 10");
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
    let stat = parse_stat_line("cpu0 10 20 30 40 50 60 70 80 90 100");
    assert_eq!(stat.user, 10);
    assert_eq!(stat.idle, 40);
    assert_eq!(stat.iowait, 50);
}

#[test]
fn parses_line_with_extra_whitespace() {
    // /proc/stat sometimes has multiple spaces after the label
    let stat = parse_stat_line("cpu   100 0 50 800 10 0 0 0 0 0");
    assert_eq!(stat.user, 100);
    assert_eq!(stat.system, 50);
    assert_eq!(stat.idle, 800);
}

#[test]
fn parses_empty_line_as_zeros() {
    let stat = parse_stat_line("cpu");
    assert_eq!(stat.user, 0);
    assert_eq!(stat.nice, 0);
    assert_eq!(stat.system, 0);
    assert_eq!(stat.idle, 0);
    assert_eq!(stat.iowait, 0);
    assert_eq!(stat.irq, 0);
    assert_eq!(stat.softirq, 0);
    assert_eq!(stat.steal, 0);
    assert_eq!(stat.guest, 0);
    assert_eq!(stat.guest_nice, 0);
}

#[test]
fn parses_partial_line_remaining_fields_are_zero() {
    // fewer than 10 fields — missing ones default to 0
    let stat = parse_stat_line("cpu 100 200 300");
    assert_eq!(stat.user, 100);
    assert_eq!(stat.nice, 200);
    assert_eq!(stat.system, 300);
    assert_eq!(stat.idle, 0);
    assert_eq!(stat.steal, 0);
}

#[test]
fn parses_non_numeric_field_as_zero() {
    let stat = parse_stat_line("cpu 100 bad 300 400 500 600 700 800 900 1000");
    assert_eq!(stat.user, 100);
    assert_eq!(stat.nice, 0); // "bad" → 0
    assert_eq!(stat.system, 300);
}

// get_total_from_stat

#[test]
fn total_sums_all_fields() {
    let stat = make_stat(10, 20, 30, 40, 50, 60, 70, 80, 90, 100);
    let (total, idle_total) = get_total_from_stat(stat);
    // total = 10+20+30+40+50+60+70+80+90+100 = 550
    assert_eq!(total, 550);
    // idle_total = idle + iowait = 40 + 50 = 90
    assert_eq!(idle_total, 90);
}

#[test]
fn total_all_zeros() {
    let stat = make_stat(0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    let (total, idle_total) = get_total_from_stat(stat);
    assert_eq!(total, 0);
    assert_eq!(idle_total, 0);
}

#[test]
fn total_only_idle() {
    let stat = make_stat(0, 0, 0, 100, 0, 0, 0, 0, 0, 0);
    let (total, idle_total) = get_total_from_stat(stat);
    assert_eq!(total, 100);
    assert_eq!(idle_total, 100);
}

#[test]
fn total_only_iowait_counts_as_idle() {
    let stat = make_stat(0, 0, 0, 0, 50, 0, 0, 0, 0, 0);
    let (total, idle_total) = get_total_from_stat(stat);
    assert_eq!(total, 50);
    assert_eq!(idle_total, 50);
}

// merge_totals

#[test]
fn merge_100_percent_usage() {
    // all delta is active, none idle
    let usage = merge_totals((0, 0), (100, 0));
    assert_eq!(usage, 100.0);
}

#[test]
fn merge_0_percent_usage() {
    // all delta is idle
    let usage = merge_totals((0, 0), (100, 100));
    assert_eq!(usage, 0.0);
}

#[test]
fn merge_50_percent_usage() {
    let usage = merge_totals((0, 0), (100, 50));
    assert_eq!(usage, 50.0);
}

#[test]
fn merge_80_percent_usage() {
    let usage = merge_totals((100, 40), (200, 60));
    // total_delta=100, idle_delta=20 → 1 - 20/100 = 80%
    assert_eq!(usage, 80.0);
}

#[test]
fn merge_with_large_real_world_values() {
    // realistic /proc/stat numbers
    let t1 = (1_000_000, 800_000);
    let t2 = (1_001_000, 800_500);
    // total_delta=1000, idle_delta=500 → 50%
    let usage = merge_totals(t1, t2);
    assert_eq!(usage, 50.0);
}

// parse → total → merge round-trip

#[test]
fn full_pipeline_produces_expected_usage() {
    let line1 = "cpu 1000 0 500 8000 500 0 0 0 0 0";
    let line2 = "cpu 1100 0 600 9000 500 0 0 0 0 0";

    let t1 = get_total_from_stat(parse_stat_line(line1));
    let t2 = get_total_from_stat(parse_stat_line(line2));
    let usage = merge_totals(t1, t2);

    // total_delta = 10100-10000 = 1700 (wait, let's calc)
    // line1 total = 1000+0+500+8000+500 = 10000, idle = 8000+500 = 8500
    // line2 total = 1100+0+600+9000+500 = 11200, idle = 9000+500 = 9500
    // total_delta = 1200, idle_delta = 1000 → 1 - 1000/1200 ≈ 16.67%
    let expected = 100.0 * (1.0 - 1000.0_f64 / 1200.0_f64);
    let diff = (usage - expected).abs();
    assert!(diff < 0.001, "expected ~{:.3} got {:.3}", expected, usage);
}
