use super::types::BatteryStat;

// helpers

fn make_bat(percentage: u64, status: &str) -> BatteryStat {
    let short_status = status.get(..3).unwrap_or(status).to_string();
    BatteryStat {
        percentage,
        status: status.to_string(),
        short_status,
    }
}

// BatteryStat construction

#[test]
fn bat_stat_stores_percentage() {
    let bat = make_bat(85, "Discharging");
    assert_eq!(bat.percentage, 85);
}

#[test]
fn bat_stat_stores_full_status() {
    let bat = make_bat(100, "Full");
    assert_eq!(bat.status, "Full");
}

#[test]
fn bat_stat_stores_short_status() {
    let bat = make_bat(42, "Discharging");
    assert_eq!(bat.short_status, "Dis");
}

// short_status slicing — mirrors io::battery_status logic

#[test]
fn short_status_is_first_three_chars_of_discharging() {
    let status = "Discharging";
    let short = status.get(..3).unwrap_or(status);
    assert_eq!(short, "Dis");
}

#[test]
fn short_status_is_first_three_chars_of_charging() {
    let status = "Charging";
    let short = status.get(..3).unwrap_or(status);
    assert_eq!(short, "Cha");
}

#[test]
fn short_status_is_first_three_chars_of_full() {
    let status = "Full";
    let short = status.get(..3).unwrap_or(status);
    assert_eq!(short, "Ful");
}

#[test]
fn short_status_falls_back_to_full_string_when_shorter_than_three() {
    // a two-character status must not panic — get(..3) returns None, fallback fires
    let status = "AC";
    let short = status.get(..3).unwrap_or(status);
    assert_eq!(short, "AC");
}

#[test]
fn short_status_on_empty_string_does_not_panic() {
    let status = "";
    let short = status.get(..3).unwrap_or(status);
    assert_eq!(short, "");
}

#[test]
fn short_status_exactly_three_chars_is_returned_as_is() {
    let status = "Low";
    let short = status.get(..3).unwrap_or(status);
    assert_eq!(short, "Low");
}

// percentage boundary values

#[test]
fn bat_at_zero_percent() {
    let bat = make_bat(0, "Discharging");
    assert_eq!(bat.percentage, 0);
}

#[test]
fn bat_at_full_percent() {
    let bat = make_bat(100, "Full");
    assert_eq!(bat.percentage, 100);
}

#[test]
fn bat_at_typical_percent() {
    let bat = make_bat(57, "Discharging");
    assert_eq!(bat.percentage, 57);
    assert_eq!(bat.short_status, "Dis");
}
