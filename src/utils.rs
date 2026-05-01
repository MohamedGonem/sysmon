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
