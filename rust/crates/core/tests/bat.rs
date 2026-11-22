use sysmon_core::bat::BatteryValues;
use sysmon_core::types::{BatteryStat, BatteryUnit};

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

// BatteryStat::from_values

#[test]
fn builds_detailed_energy_stats() {
    let values = BatteryValues {
        energy_now: Some(25_000_000),
        energy_full: Some(50_000_000),
        energy_full_design: Some(60_000_000),
        power_now: Some(7_500_000),
        voltage_now: Some(11_400_000),
        cycle_count: Some(120),
        ..Default::default()
    };
    let battery = BatteryStat::from_values(Some(50), "Discharging", values).expect("should build");

    assert_eq!(battery.percentage, 50);
    assert_eq!(battery.energy_now, Some(25_000_000));
    assert_eq!(battery.energy_full, Some(50_000_000));
    assert_eq!(battery.energy_full_design, Some(60_000_000));
    assert_eq!(battery.power_now, Some(7_500_000));
    assert_eq!(battery.voltage_now, Some(11_400_000));
    assert_eq!(battery.cycle_count, Some(120));
    assert_eq!(battery.energy_unit, BatteryUnit::Energy);
    assert!((battery.health_percentage.unwrap() - 83.333).abs() < 0.01);
}

#[test]
fn falls_back_to_energy_ratio_when_capacity_is_missing() {
    let values = BatteryValues {
        energy_now: Some(25_000_000),
        energy_full: Some(50_000_000),
        ..Default::default()
    };
    let battery = BatteryStat::from_values(None, "Charging", values).expect("should build");

    assert_eq!(battery.percentage, 50);
}

#[test]
fn builds_charge_based_stats() {
    let values = BatteryValues {
        charge_now: Some(2_000_000),
        charge_full: Some(4_000_000),
        charge_full_design: Some(5_000_000),
        current_now: Some(500_000),
        voltage_now: Some(12_000_000),
        ..Default::default()
    };
    let battery = BatteryStat::from_values(None, "Discharging", values).expect("should build");

    assert_eq!(battery.percentage, 50);
    assert_eq!(battery.energy_unit, BatteryUnit::Charge);
    assert_eq!(battery.energy_now, Some(2_000_000));
    assert_eq!(battery.power_now, Some(6_000_000));
    assert!((battery.health_percentage.unwrap() - 80.0).abs() < 0.01);
}
