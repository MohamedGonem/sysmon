struct BatteryStat {
    percentage: u64,
    status: String,
    short_status: String,
}

fn battery_status() -> BatteryStat {
    let capacity: u64 = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let full_status = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .unwrap_or_default()
        .trim()
        .to_string();
    let shortened_status = full_status.get(..3).unwrap_or(&full_status).to_string();
    BatteryStat {
        percentage: capacity,
        status: full_status,
        short_status: shortened_status,
    }
}
