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

pub fn color_based_on_percentage(
    text: String,
    percentage: f64,
    tmux: bool,
    reversed: bool,
) -> String {
    let true_percentage = if reversed { -percentage } else { percentage };

    match true_percentage as i64 {
        0..=10 | -100..=-91 => {
            if tmux {
                tmux_green_bold(&text)
            } else {
                color::green_bold(&text)
            }
        }
        11..=30 | -90..=-76 => {
            if tmux {
                tmux_green(&text)
            } else {
                color::green(&text)
            }
        }
        31..=50 | -75..=-51 => {
            if tmux {
                tmux_yellow(&text)
            } else {
                color::yellow(&text)
            }
        }
        51..=75 | -50..=-31 => {
            if tmux {
                tmux_yellow_bold(&text)
            } else {
                color::yellow_bold(&text)
            }
        }
        76..=90 | -30..=-11 => {
            if tmux {
                tmux_red(&text)
            } else {
                color::red(&text)
            }
        }
        91..=100 | -10..=-1 => {
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
