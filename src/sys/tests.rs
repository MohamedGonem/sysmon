use super::parser::{format_uptime, parse_boot_time, parse_distro, parse_uptime_secs};

// parse_uptime_secs

#[test]
fn parses_uptime_integer_seconds() {
    assert_eq!(parse_uptime_secs("3661.00 1234.56"), 3661);
}

#[test]
fn parses_uptime_fractional_seconds_truncates() {
    assert_eq!(parse_uptime_secs("90.75 45.10"), 90);
}

#[test]
fn parses_uptime_empty_content_returns_zero() {
    assert_eq!(parse_uptime_secs(""), 0);
}

#[test]
fn parses_uptime_non_numeric_returns_zero() {
    assert_eq!(parse_uptime_secs("bad content"), 0);
}

// format_uptime

#[test]
fn formats_zero_uptime() {
    assert_eq!(format_uptime(0), "0d 0h 0m 0s");
}

#[test]
fn formats_exactly_one_minute() {
    assert_eq!(format_uptime(60), "0d 0h 1m 0s");
}

#[test]
fn formats_exactly_one_hour() {
    assert_eq!(format_uptime(3600), "0d 1h 0m 0s");
}

#[test]
fn formats_exactly_one_day() {
    assert_eq!(format_uptime(86400), "1d 0h 0m 0s");
}

#[test]
fn formats_mixed_uptime() {
    // 1d 2h 3m 4s = 86400 + 7200 + 180 + 4 = 93784
    assert_eq!(format_uptime(93784), "1d 2h 3m 4s");
}

#[test]
fn formats_large_uptime() {
    // 10 days exactly
    assert_eq!(format_uptime(864000), "10d 0h 0m 0s");
}

// parse_distro

#[test]
fn parses_pretty_name_double_quoted() {
    let content = "ID=arch\nPRETTY_NAME=\"Arch Linux\"\nID_LIKE=linux\n";
    assert_eq!(parse_distro(content).as_deref(), Some("Arch Linux"));
}

#[test]
fn parses_pretty_name_unquoted() {
    let content = "PRETTY_NAME=Fedora\n";
    assert_eq!(parse_distro(content).as_deref(), Some("Fedora"));
}

#[test]
fn returns_none_when_pretty_name_absent() {
    let content = "ID=arch\nNAME=Arch\n";
    assert!(parse_distro(content).is_none());
}

#[test]
fn returns_none_on_empty_os_release() {
    assert!(parse_distro("").is_none());
}

// parse_boot_time

#[test]
fn parses_btime_from_proc_stat() {
    let content = "cpu  100 0 50 800\nbtime 1700000000\nprocesses 1234\n";
    assert_eq!(parse_boot_time(content), Some(1700000000));
}

#[test]
fn returns_none_when_btime_absent() {
    let content = "cpu  100 0 50 800\nprocesses 1234\n";
    assert!(parse_boot_time(content).is_none());
}

#[test]
fn returns_none_on_empty_stat() {
    assert!(parse_boot_time("").is_none());
}

#[test]
fn returns_none_when_btime_value_non_numeric() {
    let content = "btime bad\n";
    assert!(parse_boot_time(content).is_none());
}
