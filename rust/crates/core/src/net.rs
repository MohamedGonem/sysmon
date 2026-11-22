use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::types::{NetworkInterface, NetworkStats};

const SYS_CLASS_NET: &str = "/sys/class/net";

impl NetworkStats {
    /// Read interface counters without sampling speeds.
    pub fn read() -> Option<Self> {
        Some(Self {
            interfaces: read_interfaces()?,
        })
    }

    /// Sample counters over a window and compute per-interface transfer rates.
    pub fn sample(window: Duration) -> Option<Self> {
        let start = Instant::now();
        let earlier = Self::read()?;
        std::thread::sleep(window);
        let mut later = Self::read()?;
        let elapsed = start.elapsed().as_secs_f64().max(0.001);

        let previous: HashMap<_, _> = earlier
            .interfaces
            .into_iter()
            .map(|interface| (interface.name.clone(), interface))
            .collect();

        for interface in &mut later.interfaces {
            let Some(previous) = previous.get(&interface.name) else {
                continue;
            };

            interface.download_speed = interface
                .bytes_received
                .saturating_sub(previous.bytes_received)
                as f64
                / elapsed;
            interface.upload_speed =
                interface.bytes_sent.saturating_sub(previous.bytes_sent) as f64 / elapsed;
        }

        Some(later)
    }
}

fn read_interfaces() -> Option<Vec<NetworkInterface>> {
    let mut addresses = interface_addresses();
    let entries = fs::read_dir(SYS_CLASS_NET).ok()?;
    let mut interfaces = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let directory = entry.path();
        let statistics = directory.join("statistics");

        let counter = |file: &str| read_u64(&statistics.join(file)).unwrap_or(0);
        let flags = read_trimmed(&directory.join("flags"))
            .and_then(|flags| parse_hex(&flags))
            .unwrap_or(0);
        let operstate = read_trimmed(&directory.join("operstate")).unwrap_or_default();
        let mac_address = read_trimmed(&directory.join("address")).unwrap_or_default();

        interfaces.push(NetworkInterface {
            name: name.clone(),
            bytes_received: counter("rx_bytes"),
            bytes_sent: counter("tx_bytes"),
            download_speed: 0.0,
            upload_speed: 0.0,
            packets_received: counter("rx_packets"),
            packets_sent: counter("tx_packets"),
            errors_in: counter("rx_errors"),
            errors_out: counter("tx_errors"),
            drops_in: counter("rx_dropped"),
            drops_out: counter("tx_dropped"),
            is_up: flags & libc::IFF_UP as u64 != 0 || operstate == "up",
            mac_address: if mac_address == "00:00:00:00:00:00" {
                String::new()
            } else {
                mac_address
            },
            ip_addresses: addresses.remove(&name).unwrap_or_default(),
        });
    }

    interfaces.sort_by(|left, right| left.name.cmp(&right.name));
    Some(interfaces)
}

/// Collect IPv4 and IPv6 addresses through `getifaddrs`.
fn interface_addresses() -> HashMap<String, Vec<String>> {
    let mut addresses: HashMap<String, Vec<String>> = HashMap::new();
    let mut list: *mut libc::ifaddrs = std::ptr::null_mut();

    // SAFETY: `getifaddrs` initialises `list` on success; the matching
    // `freeifaddrs` below releases it.
    if unsafe { libc::getifaddrs(&mut list) } != 0 {
        return addresses;
    }

    let mut cursor = list;
    while !cursor.is_null() {
        // SAFETY: `cursor` is a node in the list returned by `getifaddrs`.
        let entry = unsafe { &*cursor };

        if !entry.ifa_addr.is_null() && !entry.ifa_name.is_null() {
            // SAFETY: both pointers are non-null and owned by the list.
            let family = unsafe { (*entry.ifa_addr).sa_family as i32 };
            if family == libc::AF_INET || family == libc::AF_INET6 {
                let name = unsafe { CStr::from_ptr(entry.ifa_name) }
                    .to_string_lossy()
                    .into_owned();

                if let Some(address) = numeric_address(entry.ifa_addr, family) {
                    let list = addresses.entry(name).or_default();
                    if !list.contains(&address) {
                        list.push(address);
                    }
                }
            }
        }

        // SAFETY: `ifa_next` is either null or the next node in the same list.
        cursor = unsafe { (*cursor).ifa_next };
    }

    // SAFETY: `list` was allocated by `getifaddrs` and is no longer used.
    unsafe { libc::freeifaddrs(list) };

    for list in addresses.values_mut() {
        list.sort();
    }

    addresses
}

fn numeric_address(address: *const libc::sockaddr, family: i32) -> Option<String> {
    let length = if family == libc::AF_INET {
        std::mem::size_of::<libc::sockaddr_in>()
    } else {
        std::mem::size_of::<libc::sockaddr_in6>()
    } as libc::socklen_t;
    let mut host = [0 as libc::c_char; 1025];

    // SAFETY: `address` points to a sockaddr of the family used to compute
    // `length`; `host` is a writable buffer of the supplied size.
    let result = unsafe {
        libc::getnameinfo(
            address,
            length,
            host.as_mut_ptr(),
            host.len() as libc::socklen_t,
            std::ptr::null_mut(),
            0,
            libc::NI_NUMERICHOST,
        )
    };

    if result != 0 {
        return None;
    }

    // SAFETY: `getnameinfo` succeeded, so `host` contains a NUL-terminated string.
    Some(
        unsafe { CStr::from_ptr(host.as_ptr()) }
            .to_string_lossy()
            .into_owned(),
    )
}

fn read_trimmed(path: &Path) -> Option<String> {
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim().to_string();

    (!value.is_empty()).then_some(value)
}

fn read_u64(path: &Path) -> Option<u64> {
    read_trimmed(path)?.parse().ok()
}

fn parse_hex(value: &str) -> Option<u64> {
    let value = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);

    u64::from_str_radix(value, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::parse_hex;

    #[test]
    fn parses_sysfs_hex_flags() {
        assert_eq!(parse_hex("0x1003"), Some(0x1003));
        assert_eq!(parse_hex("0X9"), Some(9));
        assert_eq!(parse_hex("3"), Some(3));
        assert_eq!(parse_hex("nope"), None);
    }
}
