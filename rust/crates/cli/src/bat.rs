use crate::utils::{color_bat, format_energy, format_power, format_voltage};
use sysmon_core::types::{BatteryStat, BatteryUnit};

pub fn print_battery(tmux: bool, detailed: bool) {
    display_battery(BatteryStat::read(), tmux, detailed);
}

fn display_battery(bat: Option<BatteryStat>, tmux: bool, detailed: bool) {
    let Some(bat) = bat else {
        println!("No battery");
        return;
    };

    let colored = color_bat(bat.percentage, tmux, &bat.status);

    if !detailed {
        println!("{} {} {}", colored[0], colored[1], colored[2]);
        return;
    }

    let unit = bat.energy_unit;
    let (now_label, full_label) = match unit {
        BatteryUnit::Energy => ("Energy Now", "Energy Full"),
        BatteryUnit::Charge => ("Charge Now", "Charge Full"),
    };
    let energy_now = optional_energy(bat.energy_now, unit);
    let energy_full = optional_energy(bat.energy_full, unit);
    let energy_full_design = optional_energy(bat.energy_full_design, unit);
    let power_now = bat
        .power_now
        .map(format_power)
        .unwrap_or_else(|| "N/A".to_string());
    let voltage_now = bat
        .voltage_now
        .map(format_voltage)
        .unwrap_or_else(|| "N/A".to_string());
    let cycle_count = bat
        .cycle_count
        .map(|cycles| cycles.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    let health = bat
        .health_percentage
        .map(|health| format!("{health:.1}%"))
        .unwrap_or_else(|| "N/A".to_string());

    println!("{}:", colored[0]);
    println!("\tStatus:\t\t{}", bat.status);
    println!("\tCapacity:\t{}", colored[2]);
    println!("\t{now_label}:\t{energy_now}");
    println!("\t{full_label}:\t{energy_full}");
    println!("\tDesign:\t\t{energy_full_design}");
    println!("\tPower:\t\t{power_now}");
    println!("\tVoltage:\t{voltage_now}");
    println!("\tCycles:\t\t{cycle_count}");
    println!("\tHealth:\t\t{health}");
}

fn optional_energy(value: Option<u64>, unit: BatteryUnit) -> String {
    value
        .map(|value| format_energy(value, unit))
        .unwrap_or_else(|| "N/A".to_string())
}
