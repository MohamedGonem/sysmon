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


pub fn color_head(text: &str, tmux: bool) -> String {
    if tmux {
        format!("#[fg=brightwhite]{}", text)
    } else {
        text.white().bold().to_string()
    }
}
