// ! need comment
pub fn read_usage_line(target: &str) -> Option<String> {
    let full_stats = std::fs::read_to_string("/proc/stat").ok()?;
    let line = full_stats.lines().find(|l| {
        l.starts_with(target)
            && l[target.len()..].starts_with(|c: char| c == ' ' || !c.is_ascii_digit())
    })?;

    Some(line.to_string())
}

pub fn get_cpu_temperature() -> Option<f64> {
    for i in 0..20 {
        let name_path = format!("/sys/class/hwmon/hwmon{}/name", i);
        let Ok(name) = std::fs::read_to_string(&name_path) else {
            continue;
        };
        let name = name.trim();

        if name == "coretemp" || name == "k10temp" {
            let temp_path = format!("/sys/class/hwmon/hwmon{}/temp1_input", i);
            let Ok(raw) = std::fs::read_to_string(&temp_path) else {
                continue;
            };
            let millidegrees: f64 = raw.trim().parse().ok()?;
            return Some(millidegrees / 1000.0);
        }
    }
    let raw = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok()?;
    let millidegrees: f64 = raw.trim().parse().ok()?;
    Some(millidegrees / 1000.0)
}

pub fn get_core_temperature(core_id: u64) -> Option<f64> {
    for i in 0..20 {
        let name_path = format!("/sys/class/hwmon/hwmon{}/name", i);
        let Ok(name) = std::fs::read_to_string(&name_path) else {
            continue;
        };
        let name = name.trim();

        if name == "coretemp" || name == "k10temp" {
            let temp_path = format!("/sys/class/hwmon/hwmon{}/temp{}_input", i, core_id + 2);
            let Ok(raw) = std::fs::read_to_string(&temp_path) else {
                continue;
            };
            let millidegree: f64 = raw.trim().parse().ok()?;
            return Some(millidegree / 1000.0);
        }
    }
    None
}

fn get_cpuinfo() -> String {
    match std::fs::read_to_string("/proc/cpuinfo") {
        Ok(s) => s.trim().to_string(),
        Err(_) => "Unknown".to_string(),
    }
}
pub fn get_cpu_flags() -> Option<Vec<String>> {
    let info = get_cpuinfo();
    let interseting = [
        "avx",
        "avx2",
        "avx512f",
        "sse4_1",
        "fma",
        "aes",
        "sgx",
        "vmx",
        "svm",
        "hypervisor",
        "ht",
    ];
    Some(
        info.lines()
            .find(|l| l.starts_with("flags"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
            .split_whitespace()
            .filter(|f| interseting.contains(f))
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    )
}

pub fn get_governor(n: &str) -> String {
    match std::fs::read_to_string(format!(
        "/sys/devices/system/cpu/cpu{n}/cpufreq/scaling_governor"
    )) {
        Ok(s) => s.trim().to_string(),
        Err(_) => "Unknown".to_string(),
    }
}

pub fn get_cpu_vendor_id() -> String {
    let info = get_cpuinfo();
    info.lines()
        .find(|l| l.starts_with("vendor_id"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}
pub fn get_cpu_model_name() -> String {
    let info = get_cpuinfo();
    info.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

pub fn get_physical_cores() -> u64 {
    let info = get_cpuinfo();
    info.lines()
        .find(|l| l.starts_with("cpu cores"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1)
}

pub fn get_logical_cores() -> u64 {
    let info = get_cpuinfo();
    info.lines().filter(|l| l.starts_with("processor")).count() as u64
}

pub fn get_frequency(path: &str) -> f64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map(|khz| khz / 1000.0)
        .unwrap_or(0.0)
}

pub fn get_leveln_cache(target: &str) -> [String; 3] {
    let info_types = ["type", "level", "size"];
    info_types.map(|info_type| {
        std::fs::read_to_string(format!(
            "/sys/devices/system/cpu/cpu0/cache/index{target}/{info_type}"
        ))
        .unwrap_or_default()
        .trim()
        .to_string()
    })
}
