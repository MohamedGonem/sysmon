use std::time::Duration;

use crate::utils::{color, color_head, format_bytes, format_rate};
use sysmon_core::types::{DiskDevice, DiskMount, DiskStats};

const SAMPLE_WINDOW: Duration = Duration::from_secs(1);

pub fn print_disks(tmux: bool, detailed: bool) {
    match DiskStats::sample(SAMPLE_WINDOW) {
        Some(stats) => display_disks(stats, tmux, detailed),
        None => eprintln!("error: could not read disk statistics"),
    }
}

fn display_disks(stats: DiskStats, tmux: bool, detailed: bool) {
    if stats.mounts.is_empty() && stats.devices.is_empty() {
        println!("No disks found");
        return;
    }

    if !detailed {
        for mount in stats.mounts {
            println!(
                "{} [{}] {} {} / {}",
                color_head(&mount.mount_point, tmux),
                mount.filesystem,
                color(mount.usage_percentage, tmux),
                format_bytes(mount.used_bytes),
                format_bytes(mount.total_bytes)
            );
        }

        for device in stats.devices {
            println!(
                "{} [{}] read {} write {}",
                color_head(&device.name, tmux),
                if device.is_rotational { "HDD" } else { "SSD" },
                format_rate(device.read_speed),
                format_rate(device.write_speed)
            );
        }
        return;
    }

    for mount in stats.mounts {
        display_mount(mount, tmux);
    }

    for device in stats.devices {
        display_device(device, tmux);
    }
}

fn display_mount(mount: DiskMount, tmux: bool) {
    println!("{}:", color_head(&mount.mount_point, tmux));
    println!("\tDevice:\t\t{}", mount.device);
    println!("\tFilesystem:\t{}", mount.filesystem);
    println!("\tTotal:\t\t{}", format_bytes(mount.total_bytes));
    println!("\tUsed:\t\t{}", format_bytes(mount.used_bytes));
    println!("\tFree:\t\t{}", format_bytes(mount.free_bytes));
    println!("\tAvailable:\t{}", format_bytes(mount.available_bytes));
    println!("\tUsage:\t\t{}", color(mount.usage_percentage, tmux));
}

fn display_device(device: DiskDevice, tmux: bool) {
    println!(
        "{} [{}]:",
        color_head(&device.name, tmux),
        if device.is_rotational { "HDD" } else { "SSD" }
    );
    println!("\tReads:\t\t{}", device.reads_completed);
    println!("\tWrites:\t\t{}", device.writes_completed);
    println!("\tRead:\t\t{}", format_bytes(device.read_bytes));
    println!("\tWritten:\t{}", format_bytes(device.write_bytes));
    println!("\tRead Speed:\t{}", format_rate(device.read_speed));
    println!("\tWrite Speed:\t{}", format_rate(device.write_speed));
}
