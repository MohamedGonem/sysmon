use colored::Colorize;
use std::f64;

pub fn color(percentage: f64, tmux: bool) -> String {
    if tmux {
        match percentage as u64 {
            0..=10 => format!("#[fg=brightgreen]{:0.1}%", percentage),
            11..=30 => format!("#[fg=green]{:0.1}%", percentage),
            31..=50 => format!("#[fg=yellow]{:0.1}%", percentage),
            51..=75 => format!("#[fg=brightyellow]{:0.1}%", percentage),
            76..=90 => format!("#[fg=red]{:0.1}%", percentage),
            _ => format!("#[fg=brightred]{:0.1}%", percentage),
        }
    } else {
        match percentage as u64 {
            0..=10 => format!("{:0.1}%", percentage).green().bold().to_string(),
            11..=30 => format!("{:0.1}%", percentage).green().to_string(),
            31..=50 => format!("{:0.1}%", percentage).yellow().to_string(),
            51..=75 => format!("{:0.1}%", percentage).yellow().bold().to_string(),
            76..=90 => format!("{:0.1}%", percentage).red().to_string(),
            _ => format!("{:0.1}%", percentage).red().to_string(),
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
            0..=10 | -100..=-91 => format!("#[fg=brightgreen]{:0.1}%", text),
            11..=30 | -90..=-76 => format!("#[fg=green]{}", text),
            31..=50 | -75..=-51 => format!("#[fg=yellow]{}", text),
            51..=75 | -50..=-31 => format!("#[fg=brightyellow]{}", text),
            76..=90 | -30..=-11 => format!("#[fg=red]{}", text),
            91..=100 | -10..=-1 => format!("#[fg=brightred]{}", text),
            _ => "".to_string(),
        }
    } else {
        match true_percentage as i64 {
            0..=10 | -100..=-91 => text.green().bold().to_string(),
            11..=30 | -90..=-76 => text.green().to_string(),
            31..=50 | -75..=-51 => text.yellow().to_string(),
            51..=75 | -50..=-31 => text.yellow().bold().to_string(),
            76..=90 | -30..=-11 => text.red().to_string(),
            91..=100 | -10..=-1 => text.red().to_string(),
            _ => "".to_string(),
        }
    }
}

pub fn color_bat(percentage: u64, tmux: bool, state: String, short_state: String) -> [String; 3] {
    let colored_state = |state: String, tmux: bool, short_state: String| -> String {
        if tmux {
            match state.as_str() {
                "Charging" => format!("#[fg=brightblue]{}", short_state),
                "Discharging" => format!("#[fg=yellow]{}", short_state),
                "Full" => format!("#[fg=brightgreen]{}", short_state),
                "Unknown" => format!("#[fg=white]{}", short_state),
                _ => "#[fg=white]Unk".to_string(),
            }
        } else {
            let right_par: String = "[".white().bold().to_string();
            let left_par: String = "]".white().bold().to_string();
            match state.as_str() {
                "Charging" => format!(
                    "{}{}{}",
                    right_par,
                    short_state.bright_blue().bold(),
                    left_par
                ),
                "Discharging" => format!("{}{}{}", right_par, short_state.yellow(), left_par),
                "Full" => format!(
                    "{}{}{}",
                    right_par,
                    short_state.bright_green().bold(),
                    left_par
                ),
                "Unknown" => format!("{}{}{}", right_par, short_state.white(), left_par),
                _ => format!("{}{}{}", right_par, short_state.white(), left_par),
            }
        }
    };
    let state = colored_state(state, tmux, short_state);

    let colored_percentage = |percentage: u64, tmux: bool| -> String {
        if tmux {
            match percentage {
                86..=100 => format!("#[fg=brightgreen]{}%", percentage),
                71..=85 => format!("#[fg=green]{}%", percentage),
                46..=70 => format!("#[fg=yellow]{}%", percentage),
                21..=45 => format!("#[fg=brightyellow]{}%", percentage),
                11..=20 => format!("#[fg=red]{}%", percentage),
                _ => format!("#[fg=brightred]{}%", percentage),
            }
        } else {
            match percentage {
                86..=100 => format!("{}%", percentage.to_string().green().bold()),
                71..=85 => format!("{}%", percentage.to_string().green()),
                46..=70 => format!("{}%", percentage.to_string().yellow()),
                21..=45 => format!("{}%", percentage.to_string().yellow().bold()),
                11..=20 => format!("{}%", percentage.to_string().red()),
                _ => format!("{}%", percentage.to_string().red().bold()),
            }
        }
    };

    let percentage = colored_percentage(percentage, tmux);
    [color_head("Bat", tmux), state, percentage]
}

pub fn color_head(text: &str, tmux: bool) -> String {
    if tmux {
        format!("#[fg=brightwhite]{}", text)
    } else {
        text.white().bold().to_string()
    }
}
