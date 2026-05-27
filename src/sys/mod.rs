mod display;
mod io;
mod parser;
mod types;

#[cfg(test)]
mod tests;

pub fn print_sys(env: String) {
    let tmux = env == "tmux";
    display::display_sys(io::get_sys_stat(), tmux);
}
