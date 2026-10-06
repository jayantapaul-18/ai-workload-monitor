use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{CoreMetrics, CpuMetrics};

pub struct CpuCollector {
    prev_totals: Vec<(u64, u64)>,
    prev_overall: Option<(u64, u64)>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            prev_totals: Vec::new(),
            prev_overall: None,
        }
    }

    pub fn collect(&mut self) -> CpuMetrics {
        let (cores_raw, overall_raw) = read_proc_stat();
        let cores = compute_core_usage(&cores_raw, &mut self.prev_totals);
        let overall_usage = compute_overall_usage(overall_raw, &mut self.prev_overall);
        let load_avg = read_load_avg();
        let frequency_mhz = read_cpu_freq_mhz();

        CpuMetrics {
            cores,
            overall_usage,
            load_avg,
            frequency_mhz,
        }
    }
}

fn read_proc_stat() -> (Vec<(u64, u64)>, (u64, u64)) {
    let content = fs::read_to_string("/proc/stat").unwrap_or_default();
    let mut cores = Vec::new();
    let mut overall = (0, 0);

    for line in content.lines() {
        if line.starts_with("cpu ") {
            overall = parse_cpu_line(line);
        } else if line.starts_with("cpu") {
            cores.push(parse_cpu_line(line));
        }
    }

    (cores, overall)
}

fn parse_cpu_line(line: &str) -> (u64, u64) {
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();

    if values.is_empty() {
        return (0, 0);
    }

    let idle = values.get(3).copied().unwrap_or(0) + values.get(4).copied().unwrap_or(0);
    let total: u64 = values.iter().sum();
    (total, idle)
}

fn compute_core_usage(raw: &[(u64, u64)], prev: &mut Vec<(u64, u64)>) -> Vec<CoreMetrics> {
    if prev.is_empty() {
        *prev = raw.to_vec();
        return raw
            .iter()
            .enumerate()
            .map(|(id, _)| CoreMetrics {
                id: id as u32,
                usage: 0.0,
                frequency_mhz: read_core_freq_mhz(id),
            })
            .collect();
    }

    let result = raw
        .iter()
        .enumerate()
        .map(|(id, &(total, idle))| {
            let (prev_total, prev_idle) = prev.get(id).copied().unwrap_or((0, 0));
            let total_delta = total.saturating_sub(prev_total);
            let idle_delta = idle.saturating_sub(prev_idle);
            let usage = if total_delta == 0 {
                0.0
            } else {
                ((total_delta - idle_delta) as f32 / total_delta as f32 * 100.0).clamp(0.0, 100.0)
            };

            CoreMetrics {
                id: id as u32,
                usage,
                frequency_mhz: read_core_freq_mhz(id),
            }
        })
        .collect();

    *prev = raw.to_vec();
    result
}

fn compute_overall_usage(raw: (u64, u64), prev: &mut Option<(u64, u64)>) -> f32 {
    if prev.is_none() {
        *prev = Some(raw);
        return 0.0;
    }

    let (prev_total, prev_idle) = prev.unwrap();
    *prev = Some(raw);

    let (total, idle) = raw;
    let total_delta = total.saturating_sub(prev_total);
    let idle_delta = idle.saturating_sub(prev_idle);

    if total_delta == 0 {
        0.0
    } else {
        ((total_delta - idle_delta) as f32 / total_delta as f32 * 100.0).clamp(0.0, 100.0)
    }
}

fn read_load_avg() -> [f32; 3] {
    let content = fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let parts: Vec<f32> = content
        .split_whitespace()
        .take(3)
        .filter_map(|v| v.parse().ok())
        .collect();

    [
        parts.first().copied().unwrap_or(0.0),
        parts.get(1).copied().unwrap_or(0.0),
        parts.get(2).copied().unwrap_or(0.0),
    ]
}

fn read_cpu_freq_mhz() -> Option<f32> {
    read_core_freq_mhz(0)
}

fn read_core_freq_mhz(core: usize) -> Option<f32> {
    let path = format!("/sys/devices/system/cpu/cpu{core}/cpufreq/scaling_cur_freq");
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|khz| khz / 1000.0)
}

impl CpuCollector {
    pub fn finalize_prev(&mut self) {
        let (cores_raw, overall_raw) = read_proc_stat();
        self.prev_totals = cores_raw;
        self.prev_overall = Some(overall_raw);
    }
}

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
