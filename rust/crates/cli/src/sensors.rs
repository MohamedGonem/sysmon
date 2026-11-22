use std::collections::BTreeMap;

use crate::utils::color_head;
use sysmon_core::types::{FanSensor, SensorKind, SensorStats, TemperatureSensor};

pub fn print_sensors(tmux: bool, detailed: bool) {
    let stats = SensorStats::read();

    if stats.temperatures.is_empty() && stats.fans.is_empty() {
        println!("No sensors found");
        return;
    }

    if detailed {
        display_detailed(stats, tmux);
    } else {
        display_summary(stats);
    }
}

fn display_summary(stats: SensorStats) {
    for sensor in stats.temperatures {
        println!(
            "{} {} {}: {:.1} C",
            kind_label(sensor.kind),
            sensor.chip,
            sensor.label,
            sensor.celsius
        );
    }

    for fan in stats.fans {
        println!("Fan {} {}: {} RPM", fan.chip, fan.label, fan.rpm);
    }
}

fn display_detailed(stats: SensorStats, tmux: bool) {
    let mut chips: BTreeMap<String, (Vec<TemperatureSensor>, Vec<FanSensor>)> = BTreeMap::new();

    for sensor in stats.temperatures {
        chips.entry(sensor.chip.clone()).or_default().0.push(sensor);
    }
    for fan in stats.fans {
        chips.entry(fan.chip.clone()).or_default().1.push(fan);
    }

    for (chip, (temperatures, fans)) in chips {
        println!("{}:", color_head(&chip, tmux));

        for sensor in temperatures {
            println!("\t{}: {:.1} C", sensor.label, sensor.celsius);
        }
        for fan in fans {
            println!("\t{}: {} RPM", fan.label, fan.rpm);
        }
    }
}

fn kind_label(kind: SensorKind) -> &'static str {
    match kind {
        SensorKind::Cpu => "CPU",
        SensorKind::Disk => "Disk",
        SensorKind::Other => "Other",
    }
}
