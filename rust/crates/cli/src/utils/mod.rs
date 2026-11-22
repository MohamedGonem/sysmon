mod color;

pub fn color(percentage: f64, tmux: bool) -> String {
    if tmux {
        #[cfg(feature = "tmux")]
        {
            match percentage as u64 {
                0..=10 => color::tmux::green_bold(&format!("{:.1}%", percentage)),
                11..=30 => color::tmux::green(&format!("{:.1}%", percentage)),
                31..=50 => color::tmux::yellow(&format!("{:.1}%", percentage)),
                51..=75 => color::tmux::yellow_bold(&format!("{:.1}%", percentage)),
                76..=90 => color::tmux::red(&format!("{:.1}%", percentage)),
                _ => color::tmux::bright_red(&format!("{:.1}%", percentage)),
            }
        }

        #[cfg(not(feature = "tmux"))]
        {
            "tmux is not loaded".to_string()
        }
    } else {
        match percentage as u64 {
            0..=10 => color::green_bold(&format!("{:.1}%", percentage)),
            11..=30 => color::green(&format!("{:.1}%", percentage)),
            31..=50 => color::yellow(&format!("{:.1}%", percentage)),
            51..=75 => color::yellow_bold(&format!("{:.1}%", percentage)),
            76..=90 => color::red(&format!("{:.1}%", percentage)),
            _ => color::bright_red(&format!("{:.1}%", percentage)),
        }
    }
}

macro_rules! define_tmux_color {
    ($name:ident, $plain:path, $tmux:path) => {
        fn $name(text: &str) -> String {
            #[cfg(feature = "tmux")]
            {
                $tmux(text)
            }

            #[cfg(not(feature = "tmux"))]
            {
                $plain(text)
            }
        }
    };
}

define_tmux_color!(tmux_green_bold, color::green_bold, color::tmux::green_bold);
define_tmux_color!(tmux_green, color::green, color::tmux::green);
define_tmux_color!(tmux_yellow, color::yellow, color::tmux::yellow);
define_tmux_color!(
    tmux_yellow_bold,
    color::yellow_bold,
    color::tmux::yellow_bold
);
define_tmux_color!(tmux_red, color::red, color::tmux::red);
define_tmux_color!(tmux_red_bold, color::red_bold, color::tmux::red_bold);
define_tmux_color!(
    tmux_bright_blue_bold,
    color::bright_blue_bold,
    color::tmux::bright_blue_bold
);
define_tmux_color!(
    tmux_bright_green_bold,
    color::bright_green_bold,
    color::tmux::bright_green_bold
);
define_tmux_color!(tmux_white_bold, color::white_bold, color::tmux::white_bold);

pub fn color_based_on_percentage(text: String, percentage: f64, tmux: bool) -> String {
    match percentage as i64 {
        0..=10 => {
            if tmux {
                tmux_green_bold(&text)
            } else {
                color::green_bold(&text)
            }
        }
        11..=30 => {
            if tmux {
                tmux_green(&text)
            } else {
                color::green(&text)
            }
        }
        31..=50 => {
            if tmux {
                tmux_yellow(&text)
            } else {
                color::yellow(&text)
            }
        }
        51..=75 => {
            if tmux {
                tmux_yellow_bold(&text)
            } else {
                color::yellow_bold(&text)
            }
        }
        76..=90 => {
            if tmux {
                tmux_red(&text)
            } else {
                color::red(&text)
            }
        }
        91..=100 => {
            if tmux {
                tmux_red_bold(&text)
            } else {
                color::red_bold(&text)
            }
        }
        _ => String::new(),
    }
}

pub fn color_bat(percentage: u64, tmux: bool, state: &str) -> [String; 3] {
    let short_state = state.get(..3).unwrap_or(state);

    let colored_state = if tmux {
        match state {
            "Charging" => tmux_bright_blue_bold(short_state),
            "Discharging" => tmux_yellow(short_state),
            "Full" => tmux_bright_green_bold(short_state),
            "Unknown" => tmux_white_bold(short_state),
            _ => tmux_white_bold(short_state),
        }
    } else {
        let left_bracket = color::white_bold("[");
        let right_bracket = color::white_bold("]");

        let colored = match state {
            "Charging" => color::bright_blue_bold(short_state),
            "Discharging" => color::yellow(short_state),
            "Full" => color::bright_green_bold(short_state),
            "Unknown" => color::white_bold(short_state),
            _ => color::white_bold(short_state),
        };

        format!("{}{}{}", left_bracket, colored, right_bracket)
    };

    let colored_percentage = match percentage {
        86..=100 => {
            if tmux {
                format!("{}%", tmux_green_bold(&percentage.to_string()))
            } else {
                format!("{}%", color::green_bold(&percentage.to_string()))
            }
        }
        71..=85 => {
            if tmux {
                format!("{}%", tmux_green(&percentage.to_string()))
            } else {
                format!("{}%", color::green(&percentage.to_string()))
            }
        }
        46..=70 => {
            if tmux {
                format!("{}%", tmux_yellow(&percentage.to_string()))
            } else {
                format!("{}%", color::yellow(&percentage.to_string()))
            }
        }
        21..=45 => {
            if tmux {
                format!("{}%", tmux_yellow_bold(&percentage.to_string()))
            } else {
                format!("{}%", color::yellow_bold(&percentage.to_string()))
            }
        }
        11..=20 => {
            if tmux {
                format!("{}%", tmux_red(&percentage.to_string()))
            } else {
                format!("{}%", color::red(&percentage.to_string()))
            }
        }
        _ => {
            if tmux {
                format!("{}%", tmux_red_bold(&percentage.to_string()))
            } else {
                format!("{}%", color::red_bold(&percentage.to_string()))
            }
        }
    };

    [color_head("Bat", tmux), colored_state, colored_percentage]
}

pub fn color_head(text: &str, tmux: bool) -> String {
    if tmux {
        tmux_white_bold(text)
    } else {
        color::white_bold(text)
    }
}

/// Format a byte count with binary units.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Format a transfer rate in bytes per second.
pub fn format_rate(bytes_per_second: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_second.max(0.0) as u64))
}

/// Format an energy or charge counter from sysfs (uWh or uAh).
pub fn format_energy(value: u64, unit: sysmon_core::types::BatteryUnit) -> String {
    match unit {
        sysmon_core::types::BatteryUnit::Energy => {
            format!("{:.1} Wh", value as f64 / 1_000_000.0)
        }
        sysmon_core::types::BatteryUnit::Charge => {
            format!("{:.2} Ah", value as f64 / 1_000_000.0)
        }
    }
}

/// Format a power counter from sysfs (uW) as watts.
pub fn format_power(microwatts: u64) -> String {
    format!("{:.2} W", microwatts as f64 / 1_000_000.0)
}

/// Format a voltage counter from sysfs (uV) as volts.
pub fn format_voltage(microvolts: u64) -> String {
    format!("{:.2} V", microvolts as f64 / 1_000_000.0)
}

/// Format a frequency in MHz.
pub fn format_frequency(mhz: f64) -> String {
    format!("{mhz:.1} MHz")
}

/// Shorten text to `max` characters, appending an ellipsis when truncated.
pub fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }

    let mut truncated: String = text.chars().take(max.saturating_sub(3)).collect();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_with_binary_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(3 * 1024 * 1024 * 1024), "3.0 GiB");
    }

    #[test]
    fn formats_rates() {
        assert_eq!(format_rate(0.0), "0 B/s");
        assert_eq!(format_rate(1536.0), "1.5 KiB/s");
    }

    #[test]
    fn truncates_long_text() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefghij", 7), "abcd...");
    }
}
