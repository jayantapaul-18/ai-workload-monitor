use std::fs;
use std::sync::OnceLock;

use super::types::SystemInfo;

static SYSTEM_INFO: OnceLock<SystemInfo> = OnceLock::new();

pub struct SystemCollector;

impl SystemCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> SystemInfo {
        SYSTEM_INFO.get_or_init(read_system_info).clone()
    }

    pub fn uptime_secs(&self) -> u64 {
        fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| s.split_whitespace().next().and_then(|v| v.parse().ok()))
            .unwrap_or(0)
    }
}

fn read_system_info() -> SystemInfo {
    let hostname = fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "localhost".into());

    let kernel = fs::read_to_string("/proc/version")
        .ok()
        .and_then(|v| v.split_whitespace().nth(2).map(str::to_string))
        .unwrap_or_else(|| "Linux".into());

    let (cpu_model, cpu_cores, cpu_threads) = read_cpu_info();
    let total_memory_bytes = read_total_memory();

    SystemInfo {
        hostname,
        os: read_os_release(),
        kernel,
        cpu_model,
        cpu_cores,
        cpu_threads,
        total_memory_bytes,
        uptime_secs: 0,
        architecture: std::env::consts::ARCH.into(),
    }
}

fn read_cpu_info() -> (String, u32, u32) {
    let content = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut model = "Unknown CPU".to_string();
    let mut core_ids = std::collections::HashSet::new();
    let mut threads = 0u32;

    for line in content.lines() {
        if line.starts_with("model name") {
            if let Some((_, value)) = line.split_once(':') {
                model = value.trim().to_string();
            }
        } else if line.starts_with("processor") {
            threads += 1;
        } else if line.starts_with("core id") {
            if let Some((_, value)) = line.split_once(':') {
                if let Ok(id) = value.trim().parse::<u32>() {
                    core_ids.insert(id);
                }
            }
        }
    }

    let cores = if core_ids.is_empty() {
        threads
    } else {
        core_ids.len() as u32
    };

    (model, cores, threads)
}

fn read_total_memory() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                if line.starts_with("MemTotal:") {
                    line.split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<u64>().ok())
                        .map(|kb| kb * 1024)
                } else {
                    None
                }
            })
        })
        .unwrap_or(0)
}

fn read_os_release() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=")
                    .map(|v| v.trim_matches('"').to_string())
            })
        })
        .unwrap_or_else(|| "Linux".into())
}
