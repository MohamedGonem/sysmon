use sysmon_core::types::SysStat;

// SysStat::parse_uptime_secs

#[test]
fn parses_uptime_integer_seconds() {
    assert_eq!(SysStat::parse_uptime_secs("3661.00 1234.56"), 3661);
}

#[test]
fn parses_uptime_fractional_seconds_truncates() {
    assert_eq!(SysStat::parse_uptime_secs("90.75 45.10"), 90);
}

#[test]
fn parses_uptime_empty_content_returns_zero() {
    assert_eq!(SysStat::parse_uptime_secs(""), 0);
}

#[test]
fn parses_uptime_non_numeric_returns_zero() {
    assert_eq!(SysStat::parse_uptime_secs("bad content"), 0);
}

// SysStat::parse_distro

#[test]
fn parses_pretty_name_double_quoted() {
    let content = "ID=arch\nPRETTY_NAME=\"Arch Linux\"\nID_LIKE=linux\n";
    assert_eq!(
        SysStat::parse_distro(content).as_deref(),
        Some("Arch Linux")
    );
}

#[test]
fn parses_pretty_name_unquoted() {
    let content = "PRETTY_NAME=Fedora\n";
    assert_eq!(SysStat::parse_distro(content).as_deref(), Some("Fedora"));
}

#[test]
fn parses_pretty_name_single_quoted() {
    let content = "PRETTY_NAME='Gentoo Linux'\n";
    assert_eq!(
        SysStat::parse_distro(content).as_deref(),
        Some("Gentoo Linux")
    );
}

#[test]
fn returns_none_when_pretty_name_absent() {
    let content = "ID=arch\nNAME=Arch\n";
    assert!(SysStat::parse_distro(content).is_none());
}

#[test]
fn returns_none_on_empty_os_release() {
    assert!(SysStat::parse_distro("").is_none());
}

// SysStat::parse_boot_time

#[test]
fn parses_btime_from_proc_stat() {
    let content = "cpu  100 0 50 800\nbtime 1700000000\nprocesses 1234\n";
    assert_eq!(SysStat::parse_boot_time(content), Some(1700000000));
}

#[test]
fn returns_none_when_btime_absent() {
    let content = "cpu  100 0 50 800\nprocesses 1234\n";
    assert!(SysStat::parse_boot_time(content).is_none());
}

#[test]
fn returns_none_on_empty_stat() {
    assert!(SysStat::parse_boot_time("").is_none());
}

#[test]
fn returns_none_when_btime_value_non_numeric() {
    let content = "btime bad\n";
    assert!(SysStat::parse_boot_time(content).is_none());
}
