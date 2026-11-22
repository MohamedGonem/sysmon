use std::time::Duration;

use crate::utils::{color_head, format_bytes, format_rate};
use sysmon_core::types::NetworkStats;

const SAMPLE_WINDOW: Duration = Duration::from_secs(1);

pub fn print_network(tmux: bool, detailed: bool) {
    match NetworkStats::sample(SAMPLE_WINDOW) {
        Some(stats) => display_network(stats, tmux, detailed),
        None => eprintln!("error: could not read network statistics"),
    }
}

fn display_network(stats: NetworkStats, tmux: bool, detailed: bool) {
    if stats.interfaces.is_empty() {
        println!("No network interfaces");
        return;
    }

    for interface in stats.interfaces {
        let state = if interface.is_up { "UP" } else { "DOWN" };
        let head = color_head(&interface.name, tmux);

        if !detailed {
            println!(
                "{head} [{state}] down {} up {}",
                format_rate(interface.download_speed),
                format_rate(interface.upload_speed)
            );
            continue;
        }

        let addresses = if interface.ip_addresses.is_empty() {
            "N/A".to_string()
        } else {
            interface.ip_addresses.join(", ")
        };
        let mac = if interface.mac_address.is_empty() {
            "N/A"
        } else {
            &interface.mac_address
        };

        println!("{head}:");
        println!("\tState:\t\t{state}");
        println!("\tMAC:\t\t{mac}");
        println!("\tIP:\t\t{addresses}");
        println!("\tReceived:\t{}", format_bytes(interface.bytes_received));
        println!("\tSent:\t\t{}", format_bytes(interface.bytes_sent));
        println!("\tDownload:\t{}", format_rate(interface.download_speed));
        println!("\tUpload:\t\t{}", format_rate(interface.upload_speed));
        println!(
            "\tPackets:\trx {} tx {}",
            interface.packets_received, interface.packets_sent
        );
        println!(
            "\tErrors:\t\tin {} out {}",
            interface.errors_in, interface.errors_out
        );
        println!(
            "\tDrops:\t\tin {} out {}",
            interface.drops_in, interface.drops_out
        );
    }
}
