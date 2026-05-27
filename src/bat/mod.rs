mod display;
mod io;
#[cfg(test)]
mod tests;
mod types;
pub fn print_battery(arg_env: String) {
    display::display_battery(io::battery_status(), arg_env);
}
