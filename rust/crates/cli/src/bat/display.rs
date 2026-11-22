use super::types;
use crate::utils::color_bat;

pub fn display_battery(bat: Option<types::BatteryStat>, arg_env: String) {
    match bat {
        Some(bat) => {
            let tmux = arg_env == "tmux";
            let bat = color_bat(bat.percentage, tmux, bat.status, bat.short_status);
            println!("{} {} {}", bat[0], bat[1], bat[2]);
        }

        _ => println!("No battery"),
    }
}
