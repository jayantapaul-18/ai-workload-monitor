use std::fs;
use std::path::Path;

use super::types::NpuMetrics;

pub struct NpuCollector;

impl NpuCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> NpuMetrics {
        let accel = Path::new("/sys/class/accel/accel0/device");

        if !accel.exists() {
            return NpuMetrics {
                available: false,
                name: "No NPU detected".into(),
                driver: String::new(),
                firmware_version: String::new(),
                power_state: String::new(),
                usage_percent: None,
            };
        }

        let driver = read_file(accel.join("uevent"))
            .and_then(|s| {
                s.lines()
                    .find_map(|l| l.strip_prefix("DRIVER=").map(str::to_string))
            })
            .unwrap_or_else(|| "amdxdna".into());

        let firmware_version =
            read_file(accel.join("fw_version")).unwrap_or_else(|| "unknown".into());
        let power_state = read_file(accel.join("power_state")).unwrap_or_else(|| "unknown".into());

        NpuMetrics {
            available: true,
            name: "AMD XDNA NPU".into(),
            driver,
            firmware_version,
            power_state,
            usage_percent: None,
        }
    }
}

fn read_file(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path.as_ref())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
