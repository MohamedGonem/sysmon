use crate::utils::color_bat;

struct BatteryStat {
    percentage: u64,
    status: String,
    short_status: String,
}

fn battery_status() -> Option<BatteryStat> {
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

    Some(BatteryStat {
        percentage: capacity,
        status: full_status,
        short_status: shortened_status,
    })
}
pub fn print_battery(arg_env: String) {
    let Some(bat) = battery_status() else {
        println!("No battery");
        return;
    };
    let tmux = arg_env == "tmux";
    let bat = color_bat(bat.percentage, tmux, bat.status, bat.short_status);
    println!("{} {} {}", bat[0], bat[1], bat[2]);
}
