use crate::types::BatteryStat;

impl BatteryStat {
    /// Read battery capacity and status from sysfs.
    pub fn read() -> Option<Self> {
        let capacity = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity").ok()?;
        let status =
            std::fs::read_to_string("/sys/class/power_supply/BAT0/status").unwrap_or_default();

        Self::parse(&capacity, &status)
    }

    /// Parse raw capacity and status values.
    pub fn parse(capacity: &str, status: &str) -> Option<Self> {
        Some(Self {
            percentage: capacity.trim().parse().ok()?,
            status: status.trim().to_string(),
        })
    }
}
