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
