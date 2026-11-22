pub struct SysStat {
    pub hostname: String,
    pub kernel: String,
    pub distro: String,
    pub arch: String,
    pub uptime_secs: u64,
    pub uptime_fmt: String,
    pub boot_time: Option<u64>,
}
