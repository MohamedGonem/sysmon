pub fn green(text: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", text)
}
pub fn green_bold(text: &str) -> String {
    format!("\x1b[1;32m{}\x1b[0m", text)
}
pub fn yellow(text: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", text)
}
pub fn yellow_bold(text: &str) -> String {
    format!("\x1b[1;33m{}\x1b[0m", text)
}
pub fn red(text: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", text)
}
pub fn red_bold(text: &str) -> String {
    format!("\x1b[1;31m{}\x1b[0m", text)
}
pub fn white_bold(text: &str) -> String {
    format!("\x1b[1;37m{}\x1b[0m", text)
}
pub fn bright_blue_bold(text: &str) -> String {
    format!("\x1b[1;94m{}\x1b[0m", text)
}
pub fn bright_green_bold(text: &str) -> String {
    format!("\x1b[1;92m{}\x1b[0m", text)
}
pub fn bright_red(text: &str) -> String {
    format!("\x1b[91m{}\x1b[0m", text)
}

#[cfg(feature = "tmux")]
pub mod tmux {
    pub fn green(text: &str) -> String {
        format!("#[fg=green]{}", text)
    }
    pub fn green_bold(text: &str) -> String {
        format!("#[fg=brightgreen]{}", text)
    }
    pub fn yellow(text: &str) -> String {
        format!("#[fg=yellow]{}", text)
    }
    pub fn yellow_bold(text: &str) -> String {
        format!("#[fg=brightyellow]{}", text)
    }
    pub fn red(text: &str) -> String {
        format!("#[fg=red]{}", text)
    }
    pub fn red_bold(text: &str) -> String {
        format!("#[fg=brightred]{}", text)
    }
    pub fn white_bold(text: &str) -> String {
        format!("#[fg=brightwhite]{}", text)
    }
    pub fn bright_blue_bold(text: &str) -> String {
        format!("#[fg=brightblue]{}", text)
    }
    pub fn bright_green_bold(text: &str) -> String {
        format!("#[fg=brightgreen]{}", text)
    }
    pub fn bright_red(text: &str) -> String {
        format!("#[fg=brightred]{}", text)
    }
}
