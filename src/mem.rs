use crate::utils::{color, color_based_on_percentage, color_head};
use std::f64;

struct MemStat {
    total_mb: u64,
    used_mb: u64,
    free_mb: u64,
    percentage: f64,
}

