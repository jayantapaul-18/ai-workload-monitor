use std::fs;

use super::types::MemoryMetrics;

pub struct MemoryCollector;

impl MemoryCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> MemoryMetrics {
        let map = read_meminfo();

        let total = map.get("MemTotal").copied().unwrap_or(0) * 1024;
        let available = map.get("MemAvailable").copied().unwrap_or(0) * 1024;
        let cached = map.get("Cached").copied().unwrap_or(0) * 1024;
        let swap_total = map.get("SwapTotal").copied().unwrap_or(0) * 1024;
        let swap_free = map.get("SwapFree").copied().unwrap_or(0) * 1024;

        let used = total.saturating_sub(available);
        let usage_percent = if total == 0 {
            0.0
        } else {
            (used as f32 / total as f32 * 100.0).clamp(0.0, 100.0)
        };

        MemoryMetrics {
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            cached_bytes: cached,
            swap_total_bytes: swap_total,
            swap_used_bytes: swap_total.saturating_sub(swap_free),
            usage_percent,
        }
    }
}

fn read_meminfo() -> std::collections::HashMap<String, u64> {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut map = std::collections::HashMap::new();

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let kb: u64 = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            map.insert(key.trim().to_string(), kb);
        }
    }

    map
}
