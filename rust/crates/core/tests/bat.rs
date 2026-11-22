use sysmon_core::types::BatteryStat;

// BatteryStat::parse

#[test]
fn parses_capacity_and_status() {
    let battery = BatteryStat::parse("85", "Discharging").expect("should parse");
    assert_eq!(battery.percentage, 85);
    assert_eq!(battery.status, "Discharging");
}

#[test]
fn trims_whitespace() {
    let battery = BatteryStat::parse(" 100\n", " Full \n").expect("should parse");
    assert_eq!(battery.percentage, 100);
    assert_eq!(battery.status, "Full");
}

#[test]
fn returns_none_for_non_numeric_capacity() {
    assert!(BatteryStat::parse("bad", "Charging").is_none());
}

#[test]
fn returns_none_for_empty_capacity() {
    assert!(BatteryStat::parse("", "Charging").is_none());
}

#[test]
fn parses_zero_percent() {
    let battery = BatteryStat::parse("0", "Discharging").expect("should parse");
    assert_eq!(battery.percentage, 0);
}

#[test]
fn parses_full_percent() {
    let battery = BatteryStat::parse("100", "Full").expect("should parse");
    assert_eq!(battery.percentage, 100);
}

#[test]
fn parses_typical_percent() {
    let battery = BatteryStat::parse("57", "Discharging").expect("should parse");
    assert_eq!(battery.percentage, 57);
}
