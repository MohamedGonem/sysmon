mod color;

use std::f64;

pub fn color(percentage: f64, tmux: bool) -> String {
    if tmux {
        match percentage as u64 {
            0..=10 => color::tmux::green_bold(&format!("{:0.1}%", percentage)),
            11..=30 => color::tmux::green(&format!("{:0.1}%", percentage)),
            31..=50 => color::tmux::yellow(&format!("{:0.1}%", percentage)),
            51..=75 => color::tmux::yellow_bold(&format!("{:0.1}%", percentage)),
            76..=90 => color::tmux::red(&format!("{:0.1}%", percentage)),
            _ => color::tmux::bright_red(&format!("{:0.1}%", percentage)),
        }
    } else {
        match percentage as u64 {
            0..=10 => color::green_bold(&format!("{:0.1}%", percentage)),
            11..=30 => color::green(&format!("{:0.1}%", percentage)),
            31..=50 => color::yellow(&format!("{:0.1}%", percentage)),
            51..=75 => color::yellow_bold(&format!("{:0.1}%", percentage)),
            76..=90 => color::red(&format!("{:0.1}%", percentage)),
            _ => color::bright_red(&format!("{:0.1}%", percentage)),
        }
    }
}

pub fn color_based_on_percentage(
    text: String,
    percentage: f64,
    tmux: bool,
    reversed: bool,
) -> String {
    let true_percentage: f64 = if reversed { -percentage } else { percentage };
    if tmux {
        match true_percentage as i64 {
            0..=10 | -100..=-91 => color::tmux::green_bold(&text),
            11..=30 | -90..=-76 => color::tmux::green(&text),
            31..=50 | -75..=-51 => color::tmux::yellow(&text),
            51..=75 | -50..=-31 => color::tmux::yellow_bold(&text),
            76..=90 | -30..=-11 => color::tmux::red(&text),
            91..=100 | -10..=-1 => color::tmux::red_bold(&text),
            _ => "".to_string(),
        }
    } else {
        match true_percentage as i64 {
            0..=10 | -100..=-91 => color::green_bold(&text),
            11..=30 | -90..=-76 => color::green(&text),
            31..=50 | -75..=-51 => color::yellow(&text),
            51..=75 | -50..=-31 => color::yellow_bold(&text),
            76..=90 | -30..=-11 => color::red(&text),
            91..=100 | -10..=-1 => color::red_bold(&text),
            _ => "".to_string(),
        }
    }
}

#[cfg(feature = "bat")]
pub fn color_bat(percentage: u64, tmux: bool, state: String, short_state: String) -> [String; 3] {
    let colored_state = |state: String, tmux: bool, short_state: String| -> String {
        if tmux {
            match state.as_str() {
                "Charging" => color::tmux::bright_blue_bold(&short_state),
                "Discharging" => color::tmux::yellow(&short_state),
                "Full" => color::tmux::bright_green_bold(&short_state),
                "Unknown" => color::tmux::white_bold(&short_state),
                _ => color::tmux::white_bold(&short_state),
            }
        } else {
            let right_par: String = color::white_bold("[");
            let left_par: String = color::white_bold("]");
            match state.as_str() {
                "Charging" => format!(
                    "{}{}{}",
                    right_par,
                    color::bright_blue_bold(&short_state),
                    left_par
                ),
                "Discharging" => {
                    format!("{}{}{}", right_par, color::yellow(&short_state), left_par)
                }
                "Full" => format!(
                    "{}{}{}",
                    right_par,
                    color::bright_green_bold(&short_state),
                    left_par
                ),
                "Unknown" => format!(
                    "{}{}{}",
                    right_par,
                    color::white_bold(&short_state),
                    left_par
                ),
                _ => format!(
                    "{}{}{}",
                    right_par,
                    color::white_bold(&short_state),
                    left_par
                ),
            }
        }
    };
    let state = colored_state(state, tmux, short_state);

    let colored_percentage = |percentage: u64, tmux: bool| -> String {
        if tmux {
            match percentage {
                86..=100 => format!("{}%", color::tmux::green_bold(&percentage.to_string())),
                71..=85 => format!("{}%", color::tmux::green(&percentage.to_string())),
                46..=70 => format!("{}%", color::tmux::yellow(&percentage.to_string())),
                21..=45 => format!("{}%", color::tmux::yellow_bold(&percentage.to_string())),
                11..=20 => format!("{}%", color::tmux::red(&percentage.to_string())),
                _ => format!("{}%", color::tmux::red_bold(&percentage.to_string())),
            }
        } else {
            match percentage {
                86..=100 => format!("{}%", color::green_bold(&percentage.to_string())),
                71..=85 => format!("{}%", color::green(&percentage.to_string())),
                46..=70 => format!("{}%", color::yellow(&percentage.to_string())),
                21..=45 => format!("{}%", color::yellow_bold(&percentage.to_string())),
                11..=20 => format!("{}%", color::red(&percentage.to_string())),
                _ => format!("{}%", color::red_bold(&percentage.to_string())),
            }
        }
    };

    let percentage = colored_percentage(percentage, tmux);
    [color_head("Bat", tmux), state, percentage]
}

pub fn color_head(text: &str, tmux: bool) -> String {
    if tmux {
        color::tmux::white_bold(text)
    } else {
        color::white_bold(text)
    }
}
