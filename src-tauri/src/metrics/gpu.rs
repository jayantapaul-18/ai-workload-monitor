use std::fs;
use std::path::{Path, PathBuf};

use super::types::GpuMetrics;

pub struct GpuCollector;

impl GpuCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) -> GpuMetrics {
        if let Some(metrics) = collect_nvidia_gpu() {
            return metrics;
        }

        let card = find_amdgpu_card();

        let Some(card_path) = card else {
            return GpuMetrics {
                available: false,
                name: "No GPU detected".into(),
                usage_percent: 0.0,
                temperature_c: None,
                power_watts: None,
                frequency_mhz: None,
                vram_used_bytes: None,
                vram_total_bytes: None,
            };
        };

        let name = read_trimmed(&card_path.join("product_name"))
            .or_else(|| read_trimmed(&card_path.join("product_number")))
            .unwrap_or_else(|| "AMD Radeon GPU".into());
        let usage_percent = read_f32_file(&card_path.join("gpu_busy_percent")).unwrap_or(0.0);
        let hwmon = find_hwmon(&card_path);

        let temperature_c = hwmon
            .as_ref()
            .and_then(|h| read_f32_file(&h.join("temp1_input")))
            .map(|millideg| millideg / 1000.0);

        let power_watts = hwmon
            .as_ref()
            .and_then(|h| read_f32_file(&h.join("power1_average")))
            .or_else(|| {
                hwmon
                    .as_ref()
                    .and_then(|h| read_f32_file(&h.join("power1_input")))
            })
            .map(|microwatts| microwatts / 1_000_000.0);

        let frequency_mhz = hwmon
            .as_ref()
            .and_then(|h| read_f32_file(&h.join("freq1_input")))
            .map(|hz| hz / 1_000_000.0);

        let vram_used_bytes = read_u64_file(&card_path.join("mem_info_vram_used"));
        let vram_total_bytes = read_u64_file(&card_path.join("mem_info_vram_total"));

        GpuMetrics {
            available: true,
            name,
            usage_percent,
            temperature_c,
            power_watts,
            frequency_mhz,
            vram_used_bytes,
            vram_total_bytes,
        }
    }
}

fn find_amdgpu_card() -> Option<PathBuf> {
    let drm = Path::new("/sys/class/drm");
    let entries = fs::read_dir(drm).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let device = entry.path().join("device");
        let vendor = read_trimmed(&device.join("vendor")).unwrap_or_default();
        if vendor == "0x1002" || vendor == "0x1022" {
            return Some(device);
        }
    }

    None
}

fn find_hwmon(device: &Path) -> Option<PathBuf> {
    let hwmon_dir = device.join("hwmon");
    let entries: Vec<PathBuf> = fs::read_dir(&hwmon_dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();

    for path in &entries {
        if read_trimmed(&path.join("name"))
            .map(|n| n.contains("amdgpu") || n.contains("radeon"))
            .unwrap_or(false)
        {
            return Some(path.clone());
        }
    }

    entries.into_iter().next()
}

fn read_f32_file(path: &Path) -> Option<f32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn read_u64_file(path: &Path) -> Option<u64> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn collect_nvidia_gpu() -> Option<GpuMetrics> {
    if !has_nvidia_device() {
        return None;
    }

    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,temperature.gpu,power.draw,clocks.current.graphics,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    parse_nvidia_smi_line(line)
}

fn has_nvidia_device() -> bool {
    Path::new("/proc/driver/nvidia/version").exists()
        || Path::new("/dev/nvidia0").exists()
        || find_nvidia_drm_card().is_some()
}

fn find_nvidia_drm_card() -> Option<PathBuf> {
    let drm = Path::new("/sys/class/drm");
    let entries = fs::read_dir(drm).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let device = entry.path().join("device");
        let vendor = read_trimmed(&device.join("vendor")).unwrap_or_default();
        if vendor == "0x10de" {
            return Some(device);
        }
    }

    None
}

pub fn parse_nvidia_smi_line(line: &str) -> Option<GpuMetrics> {
    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.len() < 7 {
        return None;
    }

    let name = parts[0].to_string();
    let usage_percent = parts[1].parse::<f32>().unwrap_or(0.0);
    let temperature_c = parts[2].parse::<f32>().ok();
    let power_watts = parts[3].parse::<f32>().ok();
    let frequency_mhz = parts[4].parse::<f32>().ok();
    let vram_used_bytes = parts[5].parse::<u64>().ok().map(|mb| mb * 1024 * 1024);
    let vram_total_bytes = parts[6].parse::<u64>().ok().map(|mb| mb * 1024 * 1024);

    Some(GpuMetrics {
        available: true,
        name,
        usage_percent,
        temperature_c,
        power_watts,
        frequency_mhz,
        vram_used_bytes,
        vram_total_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nvidia_smi_csv() {
        let line = "NVIDIA GeForce RTX 4090, 38, 64, 185.4, 2520, 8192, 24576";
        let gpu = parse_nvidia_smi_line(line).expect("parses nvidia-smi line");
        assert_eq!(gpu.name, "NVIDIA GeForce RTX 4090");
        assert_eq!(gpu.usage_percent, 38.0);
        assert_eq!(gpu.temperature_c, Some(64.0));
        assert_eq!(gpu.power_watts, Some(185.4));
        assert_eq!(gpu.frequency_mhz, Some(2520.0));
        assert_eq!(gpu.vram_used_bytes, Some(8192 * 1024 * 1024));
        assert_eq!(gpu.vram_total_bytes, Some(24576 * 1024 * 1024));
        assert!(gpu.available);
    }
}
