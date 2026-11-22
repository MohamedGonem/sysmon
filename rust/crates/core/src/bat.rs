use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{BatteryStat, BatteryUnit};

/// Raw sysfs battery counters.
///
/// Kernels expose either `energy_*` (uWh) or `charge_*` (uAh) depending on the
/// battery firmware. `BatteryStat::from_values` normalises both into one unit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatteryValues {
    pub energy_now: Option<u64>,
    pub energy_full: Option<u64>,
    pub energy_full_design: Option<u64>,
    pub power_now: Option<u64>,
    pub voltage_now: Option<u64>,
    pub cycle_count: Option<u64>,
    pub charge_now: Option<u64>,
    pub charge_full: Option<u64>,
    pub charge_full_design: Option<u64>,
    pub current_now: Option<u64>,
}

impl BatteryStat {
    /// Read the first battery exposed by sysfs.
    pub fn read() -> Option<Self> {
        let directory = battery_directory()?;

        let capacity = read_number(&directory, "capacity");
        let status = read_trimmed(&directory, "status").unwrap_or_else(|| "Unknown".to_string());
        let values = BatteryValues {
            energy_now: read_number(&directory, "energy_now"),
            energy_full: read_number(&directory, "energy_full"),
            energy_full_design: read_number(&directory, "energy_full_design"),
            power_now: read_magnitude(&directory, "power_now"),
            voltage_now: read_number(&directory, "voltage_now"),
            cycle_count: read_number(&directory, "cycle_count"),
            charge_now: read_number(&directory, "charge_now"),
            charge_full: read_number(&directory, "charge_full"),
            charge_full_design: read_number(&directory, "charge_full_design"),
            current_now: read_magnitude(&directory, "current_now"),
        };

        Self::from_values(capacity, &status, values)
    }

    /// Parse the two values every battery exposes.
    ///
    /// This keeps the simple API used by callers that only have `capacity` and
    /// `status`; the richer fields are left empty.
    pub fn parse(capacity: &str, status: &str) -> Option<Self> {
        let percentage = capacity.trim().parse::<u64>().ok()?;

        Some(Self {
            percentage: percentage.min(100),
            status: status.trim().to_string(),
            energy_now: None,
            energy_full: None,
            energy_full_design: None,
            power_now: None,
            voltage_now: None,
            cycle_count: None,
            health_percentage: None,
            energy_unit: BatteryUnit::Energy,
        })
    }

    /// Build a battery stat from optional sysfs counters.
    ///
    /// Percentage prefers `capacity`; if that file is missing, the energy or
    /// charge ratio is used. Power falls back to `current_now * voltage_now`.
    pub fn from_values(capacity: Option<u64>, status: &str, values: BatteryValues) -> Option<Self> {
        let has_energy = values.energy_now.is_some()
            || values.energy_full.is_some()
            || values.energy_full_design.is_some();
        let has_charge = values.charge_now.is_some()
            || values.charge_full.is_some()
            || values.charge_full_design.is_some();

        let energy_unit = if has_energy {
            BatteryUnit::Energy
        } else if has_charge {
            BatteryUnit::Charge
        } else {
            BatteryUnit::Energy
        };

        let (energy_now, energy_full, energy_full_design) = match energy_unit {
            BatteryUnit::Energy => (
                values.energy_now,
                values.energy_full,
                values.energy_full_design,
            ),
            BatteryUnit::Charge => (
                values.charge_now,
                values.charge_full,
                values.charge_full_design,
            ),
        };

        let percentage = capacity.or_else(|| ratio_percentage(energy_now, energy_full))?;

        let power_now = values.power_now.or_else(|| {
            let current = values.current_now?;
            let voltage = values.voltage_now?;
            // uA * uV = pW, so divide by 1_000_000 to get uW.
            let microwatts = (current as u128 * voltage as u128) / 1_000_000;
            Some(microwatts.min(u64::MAX as u128) as u64)
        });

        let health_percentage = match (energy_full, energy_full_design) {
            (Some(full), Some(design)) if design > 0 => Some(full as f64 * 100.0 / design as f64),
            _ => None,
        };

        Some(Self {
            percentage: percentage.min(100),
            status: status.trim().to_string(),
            energy_now,
            energy_full,
            energy_full_design,
            power_now,
            voltage_now: values.voltage_now,
            cycle_count: values.cycle_count,
            health_percentage,
            energy_unit,
        })
    }
}

fn ratio_percentage(now: Option<u64>, full: Option<u64>) -> Option<u64> {
    let now = now?;
    let full = full?;
    if full == 0 {
        return None;
    }

    Some((now as f64 * 100.0 / full as f64).round() as u64)
}

fn battery_directory() -> Option<PathBuf> {
    let base = Path::new("/sys/class/power_supply");
    let entries = fs::read_dir(base).ok()?;
    let mut fallback = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if read_trimmed(&path, "type").as_deref() != Some("Battery") {
            continue;
        }

        if path.file_name().and_then(|name| name.to_str()) == Some("BAT0") {
            return Some(path);
        }

        fallback.get_or_insert(path);
    }

    fallback
}

fn read_trimmed(directory: &Path, file: &str) -> Option<String> {
    let value = fs::read_to_string(directory.join(file)).ok()?;
    let value = value.trim().to_string();

    (!value.is_empty()).then_some(value)
}

fn read_number(directory: &Path, file: &str) -> Option<u64> {
    read_trimmed(directory, file)?.parse().ok()
}

fn read_magnitude(directory: &Path, file: &str) -> Option<u64> {
    let value: i64 = read_trimmed(directory, file)?.parse().ok()?;

    Some(value.unsigned_abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> BatteryValues {
        BatteryValues {
            energy_now: Some(25_000_000),
            energy_full: Some(50_000_000),
            energy_full_design: Some(60_000_000),
            power_now: Some(7_500_000),
            voltage_now: Some(11_400_000),
            cycle_count: Some(120),
            ..Default::default()
        }
    }

    #[test]
    fn computes_health_and_keeps_energy_fields() {
        let battery = BatteryStat::from_values(Some(50), "Discharging", values()).unwrap();

        assert_eq!(battery.percentage, 50);
        assert_eq!(battery.energy_now, Some(25_000_000));
        assert_eq!(battery.energy_full, Some(50_000_000));
        assert_eq!(battery.energy_full_design, Some(60_000_000));
        assert_eq!(battery.power_now, Some(7_500_000));
        assert_eq!(battery.voltage_now, Some(11_400_000));
        assert_eq!(battery.cycle_count, Some(120));
        assert_eq!(battery.energy_unit, BatteryUnit::Energy);
        assert!((battery.health_percentage.unwrap() - 83.333).abs() < 0.01);
    }

    #[test]
    fn falls_back_to_energy_ratio() {
        let battery = BatteryStat::from_values(None, "Charging", values()).unwrap();

        assert_eq!(battery.percentage, 50);
    }

    #[test]
    fn supports_charge_counters() {
        let values = BatteryValues {
            charge_now: Some(2_000_000),
            charge_full: Some(4_000_000),
            charge_full_design: Some(5_000_000),
            current_now: Some(500_000),
            voltage_now: Some(12_000_000),
            ..Default::default()
        };
        let battery = BatteryStat::from_values(None, "Discharging", values).unwrap();

        assert_eq!(battery.percentage, 50);
        assert_eq!(battery.energy_unit, BatteryUnit::Charge);
        assert_eq!(battery.energy_now, Some(2_000_000));
        assert_eq!(battery.power_now, Some(6_000_000));
        assert!((battery.health_percentage.unwrap() - 80.0).abs() < 0.01);
    }

    #[test]
    fn clamps_percentage_to_100() {
        let battery =
            BatteryStat::from_values(Some(150), "Full", BatteryValues::default()).unwrap();

        assert_eq!(battery.percentage, 100);
    }

    #[test]
    fn requires_a_percentage_source() {
        assert!(BatteryStat::from_values(None, "Unknown", BatteryValues::default()).is_none());
    }
}
