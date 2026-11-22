use std::fs;
use std::path::Path;

use crate::types::{FanSensor, SensorKind, SensorStats, TemperatureSensor};

impl SensorStats {
    /// Read every hwmon temperature and fan sensor.
    pub fn read() -> Self {
        let mut temperatures = Vec::new();
        let mut fans = Vec::new();
        let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
            return Self { temperatures, fans };
        };

        let mut paths: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
        paths.sort();

        for path in paths {
            let Some(chip) = read_trimmed(&path.join("name")) else {
                continue;
            };
            let kind = sensor_kind(&chip);

            temperatures.extend(read_temperatures(&path, &chip, kind));
            fans.extend(read_fans(&path, &chip));
        }

        temperatures.sort_by(|left, right| {
            left.chip
                .cmp(&right.chip)
                .then_with(|| left.label.cmp(&right.label))
        });
        fans.sort_by(|left, right| {
            left.chip
                .cmp(&right.chip)
                .then_with(|| left.label.cmp(&right.label))
        });

        Self { temperatures, fans }
    }
}

fn read_temperatures(directory: &Path, chip: &str, kind: SensorKind) -> Vec<TemperatureSensor> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();

    files
        .into_iter()
        .filter_map(|file| {
            let prefix = file.strip_suffix("_input")?;
            if !prefix.starts_with("temp") {
                return None;
            }

            let raw = read_trimmed(&directory.join(&file))?;
            let millidegrees = raw.parse::<f64>().ok()?;
            let label = read_trimmed(&directory.join(format!("{prefix}_label")))
                .unwrap_or_else(|| prefix.to_string());

            Some(TemperatureSensor {
                chip: chip.to_string(),
                label,
                celsius: millidegrees / 1000.0,
                kind,
            })
        })
        .collect()
}

fn read_fans(directory: &Path, chip: &str) -> Vec<FanSensor> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();

    files
        .into_iter()
        .filter_map(|file| {
            let prefix = file.strip_suffix("_input")?;
            if !prefix.starts_with("fan") {
                return None;
            }

            let rpm = read_trimmed(&directory.join(&file))?.parse().ok()?;
            let label = read_trimmed(&directory.join(format!("{prefix}_label")))
                .unwrap_or_else(|| prefix.to_string());

            Some(FanSensor {
                chip: chip.to_string(),
                label,
                rpm,
            })
        })
        .collect()
}

fn sensor_kind(chip: &str) -> SensorKind {
    let chip = chip.to_ascii_lowercase();

    if chip.contains("coretemp")
        || chip.contains("k10temp")
        || chip.contains("zenpower")
        || chip.contains("cpu")
    {
        SensorKind::Cpu
    } else if chip.contains("drivetemp") || chip.contains("nvme") {
        SensorKind::Disk
    } else {
        SensorKind::Other
    }
}

fn read_trimmed(path: &Path) -> Option<String> {
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim().to_string();

    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::sensor_kind;
    use crate::types::SensorKind;

    #[test]
    fn classifies_cpu_sensors() {
        assert_eq!(sensor_kind("coretemp"), SensorKind::Cpu);
        assert_eq!(sensor_kind("k10temp"), SensorKind::Cpu);
        assert_eq!(sensor_kind("cpu_thermal"), SensorKind::Cpu);
    }

    #[test]
    fn classifies_disk_sensors() {
        assert_eq!(sensor_kind("drivetemp"), SensorKind::Disk);
        assert_eq!(sensor_kind("nvme"), SensorKind::Disk);
    }

    #[test]
    fn classifies_everything_else() {
        assert_eq!(sensor_kind("thinkpad"), SensorKind::Other);
        assert_eq!(sensor_kind("acpitz"), SensorKind::Other);
    }
}
