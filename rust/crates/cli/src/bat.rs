use crate::utils::color_bat;
use sysmon_core::types::BatteryStat;

pub fn print_battery(tmux: bool) {
    display_battery(BatteryStat::read(), tmux);
}

fn display_battery(bat: Option<BatteryStat>, tmux: bool) {
    match bat {
        Some(bat) => {
            let colored = color_bat(bat.percentage, tmux, &bat.status);
            println!("{} {} {}", colored[0], colored[1], colored[2]);
        }
        None => println!("No battery"),
    }
}
