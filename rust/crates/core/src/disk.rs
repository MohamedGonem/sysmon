use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::types::{DiskDevice, DiskMount, DiskStats};

const SECTOR_SIZE: u64 = 512;

impl DiskStats {
    /// Read mounts and block-device counters without sampling speeds.
    pub fn read() -> Option<Self> {
        Some(Self {
            mounts: read_mounts(),
            devices: read_devices(),
        })
    }

    /// Sample block-device counters over a window and compute I/O rates.
    pub fn sample(window: Duration) -> Option<Self> {
        let start = Instant::now();
        let earlier = Self::read()?;
        std::thread::sleep(window);
        let mut later = Self::read()?;
        let elapsed = start.elapsed().as_secs_f64().max(0.001);

        let previous: HashMap<_, _> = earlier
            .devices
            .into_iter()
            .map(|device| (device.name.clone(), device))
            .collect();

        for device in &mut later.devices {
            let Some(previous) = previous.get(&device.name) else {
                continue;
            };

            device.read_speed =
                device.read_bytes.saturating_sub(previous.read_bytes) as f64 / elapsed;
            device.write_speed =
                device.write_bytes.saturating_sub(previous.write_bytes) as f64 / elapsed;
        }

        Some(later)
    }
}

fn read_mounts() -> Vec<DiskMount> {
    let Ok(content) = fs::read_to_string("/proc/self/mountinfo") else {
        return Vec::new();
    };

    let mut mounts: Vec<_> = content.lines().filter_map(parse_mountinfo_line).collect();
    mounts.sort_by(|left, right| left.mount_point.cmp(&right.mount_point));
    mounts
}

fn parse_mountinfo_line(line: &str) -> Option<DiskMount> {
    let fields: Vec<_> = line.split_whitespace().collect();
    let separator = fields.iter().position(|field| *field == "-")?;

    if fields.len() < separator + 3 {
        return None;
    }

    let mount_point = unescape_mount(fields.get(4)?);
    let filesystem = fields[separator + 1].to_string();
    let device = unescape_mount(fields[separator + 2]);

    if !device.starts_with("/dev/") {
        return None;
    }

    let (total_bytes, used_bytes, free_bytes, available_bytes) = mount_space(&mount_point)?;
    let usage_percentage = if used_bytes.saturating_add(available_bytes) == 0 {
        0.0
    } else {
        used_bytes as f64 * 100.0 / used_bytes.saturating_add(available_bytes) as f64
    };

    Some(DiskMount {
        device,
        mount_point,
        filesystem,
        total_bytes,
        used_bytes,
        free_bytes,
        available_bytes,
        usage_percentage,
    })
}

fn mount_space(path: &str) -> Option<(u64, u64, u64, u64)> {
    let path = CString::new(path).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();

    // SAFETY: `path` is a valid NUL-terminated string and `stat` points to
    // writable memory for the duration of the call.
    if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return None;
    }

    // SAFETY: `statvfs` returned success, so `stat` was initialised.
    let stat = unsafe { stat.assume_init() };
    let block_size = if stat.f_frsize > 0 {
        stat.f_frsize
    } else {
        stat.f_bsize
    };

    let total = stat.f_blocks * block_size;
    let free = stat.f_bfree * block_size;
    let available = stat.f_bavail * block_size;
    let used = total.saturating_sub(free);

    Some((total, used, free, available))
}

fn read_devices() -> Vec<DiskDevice> {
    let Ok(content) = fs::read_to_string("/proc/diskstats") else {
        return Vec::new();
    };

    let mut devices: Vec<_> = content
        .lines()
        .filter_map(parse_diskstats_line)
        .filter(|device| Path::new("/sys/block").join(&device.name).exists())
        .map(|device| DiskDevice {
            is_rotational: read_trimmed(
                &Path::new("/sys/block")
                    .join(&device.name)
                    .join("queue/rotational"),
            )
            .as_deref()
                == Some("1"),
            ..device
        })
        .collect();

    devices.sort_by(|left, right| left.name.cmp(&right.name));
    devices
}

fn parse_diskstats_line(line: &str) -> Option<DiskDevice> {
    let fields: Vec<_> = line.split_whitespace().collect();

    if fields.len() < 10 {
        return None;
    }

    let name = fields[2].to_string();
    let reads_completed = fields[3].parse().ok()?;
    let sectors_read = fields[5].parse::<u64>().ok()?;
    let writes_completed = fields[7].parse().ok()?;
    let sectors_written = fields[9].parse::<u64>().ok()?;

    Some(DiskDevice {
        name,
        reads_completed,
        writes_completed,
        read_bytes: sectors_read.saturating_mul(SECTOR_SIZE),
        write_bytes: sectors_written.saturating_mul(SECTOR_SIZE),
        read_speed: 0.0,
        write_speed: 0.0,
        is_rotational: false,
    })
}

/// Decode the octal escapes (`\040` for spaces, for example) used by mountinfo.
fn unescape_mount(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\'
            && index + 3 < bytes.len()
            && let Ok(digits) = std::str::from_utf8(&bytes[index + 1..index + 4])
            && let Ok(code) = u8::from_str_radix(digits, 8)
        {
            result.push(code);
            index += 4;
            continue;
        }

        result.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&result).into_owned()
}

fn read_trimmed(path: &Path) -> Option<String> {
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim().to_string();

    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{parse_diskstats_line, unescape_mount};

    #[test]
    fn decodes_mountinfo_escapes() {
        assert_eq!(unescape_mount("/mnt/my\\040disk"), "/mnt/my disk");
        assert_eq!(unescape_mount("/mnt/tab\\011name"), "/mnt/tab\tname");
        assert_eq!(unescape_mount("/plain"), "/plain");
    }

    #[test]
    fn parses_diskstats_counters() {
        let line = "   8       0 sda 12345 678 901234 5678 23456 789 123456 6789 0 1234 5678";
        let device = parse_diskstats_line(line).expect("should parse");

        assert_eq!(device.name, "sda");
        assert_eq!(device.reads_completed, 12345);
        assert_eq!(device.writes_completed, 23456);
        assert_eq!(device.read_bytes, 901234 * 512);
        assert_eq!(device.write_bytes, 123456 * 512);
    }

    #[test]
    fn rejects_short_diskstats_lines() {
        assert!(parse_diskstats_line("8 0 sda 1 2 3").is_none());
    }
}
