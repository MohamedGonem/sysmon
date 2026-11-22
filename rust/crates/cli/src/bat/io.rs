use super::types;

pub fn battery_status() -> Option<types::BatteryStat> {
    let capacity: u64 = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()?
        .trim()
        .parse()
        .ok()?;

    let full_status = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .unwrap_or_default()
        .trim()
        .to_string();

    let shortened_status = full_status.get(..3).unwrap_or(&full_status).to_string();

    Some(types::BatteryStat {
        percentage: capacity,
        status: full_status,
        short_status: shortened_status,
    })
}
