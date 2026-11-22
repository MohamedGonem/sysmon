pub struct MemStat {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub percentage: f64,
    pub is_active: Option<bool>,
}
