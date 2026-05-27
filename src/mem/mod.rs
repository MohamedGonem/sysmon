mod display;
mod io;
mod types;

pub fn print_mem(env: String, mode: String, detailed: bool) {
    let Some((mem_stat, swap_stat)) = io::get_mem_usage() else {
        eprintln!("error: could not read /proc/meminfo");
        return;
    };
    let tmux = env == "tmux";

    match &mode as &str {
        "swap" => display::display_swap(swap_stat, tmux, detailed),
        "memory" => display::display_mem(mem_stat, tmux, detailed),
        "all" => {
            display::display_mem(mem_stat, tmux, detailed);
            display::display_swap(swap_stat, tmux, detailed);
        }
        &_ => println!("Invalid choice"),
    }
}
