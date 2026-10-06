use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static PROFILE: OnceLock<HardwareProfile> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub platform: PlatformSpec,
    pub cpu: CpuSpec,
    pub gpu: GpuSpec,
    pub npu: NpuSpec,
    pub memory: MemorySpec,
    pub storage: Vec<StorageDevice>,
    pub app: AppSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSpec {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub architecture: String,
    pub desktop: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSpec {
    pub model: String,
    pub vendor: String,
    pub cores: u32,
    pub threads: u32,
    pub max_frequency_mhz: Option<u32>,
    pub cache_l3_kb: Option<u32>,
    pub flags: Vec<String>,
    pub virtualization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSpec {
    pub detected: bool,
    pub name: String,
    pub driver: String,
    pub pci_address: String,
    pub vram_bytes: Option<u64>,
    pub render_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuSpec {
    pub detected: bool,
    pub name: String,
    pub driver: String,
    pub firmware: String,
    pub pci_address: String,
    pub power_state: String,
    pub utilization_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpec {
    pub total_bytes: u64,
    pub swap_bytes: u64,
    pub channels: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub name: String,
    pub size_bytes: u64,
    pub model: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSpec {
    pub name: String,
    pub version: String,
    pub identifier: String,
}

pub fn get_hardware_profile() -> HardwareProfile {
    PROFILE.get_or_init(collect_hardware_profile).clone()
}

fn collect_hardware_profile() -> HardwareProfile {
    HardwareProfile {
        platform: read_platform(),
        cpu: read_cpu_spec(),
        gpu: read_gpu_spec(),
        npu: read_npu_spec(),
        memory: read_memory_spec(),
        storage: read_storage(),
        app: AppSpec {
            name: "AI Workload Monitor".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            identifier: "com.jayantapaul.ai-workload-monitor".into(),
        },
    }
}

fn read_platform() -> PlatformSpec {
    PlatformSpec {
        hostname: read_file("/etc/hostname").unwrap_or_else(|| "localhost".into()),
        os: read_os_release(),
        kernel: fs::read_to_string("/proc/version")
            .ok()
            .and_then(|v| v.split_whitespace().nth(2).map(str::to_string))
            .unwrap_or_else(|| "Linux".into()),
        architecture: std::env::consts::ARCH.into(),
        desktop: std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Unknown".into()),
    }
}

fn read_cpu_spec() -> CpuSpec {
    let content = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut model = "Unknown CPU".to_string();
    let mut vendor = String::new();
    let mut core_ids = std::collections::HashSet::new();
    let mut threads = 0u32;
    let mut cache_l3 = None;
    let mut flags: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim();
            match key.trim() {
                "model name" => model = value.to_string(),
                "vendor_id" => vendor = value.to_string(),
                "processor" => threads += 1,
                "core id" => {
                    if let Ok(id) = value.parse::<u32>() {
                        core_ids.insert(id);
                    }
                }
                "cache size" if cache_l3.is_none() => {
                    if let Some(kb) = value.split_whitespace().next().and_then(|s| s.parse().ok()) {
                        cache_l3 = Some(kb);
                    }
                }
                "flags" => {
                    flags = value
                        .split_whitespace()
                        .filter(|f| is_notable_cpu_flag(f))
                        .take(12)
                        .map(str::to_string)
                        .collect();
                }
                _ => {}
            }
        }
    }

    let cores = if core_ids.is_empty() {
        threads
    } else {
        core_ids.len() as u32
    };

    let max_frequency_mhz = (0..threads)
        .filter_map(|i| {
            read_file(&format!(
                "/sys/devices/system/cpu/cpu{i}/cpufreq/cpuinfo_max_freq"
            ))
            .and_then(|s| s.parse::<u32>().ok())
            .map(|khz| khz / 1000)
        })
        .max();

    let virtualization = if content.contains("svm") {
        "AMD-V".into()
    } else if content.contains("vmx") {
        "Intel VT-x".into()
    } else {
        "Not reported".into()
    };

    CpuSpec {
        model,
        vendor,
        cores,
        threads,
        max_frequency_mhz,
        cache_l3_kb: cache_l3,
        flags,
        virtualization,
    }
}

fn is_notable_cpu_flag(flag: &str) -> bool {
    matches!(
        flag,
        "avx" | "avx2" | "avx512f" | "avx512_bf16" | "sse4_2" | "aes" | "sha_ni" | "fma"
    ) || flag.starts_with("avx512")
}

fn read_gpu_spec() -> GpuSpec {
    let drm = Path::new("/sys/class/drm");
    let mut spec = GpuSpec {
        detected: false,
        name: "Not detected".into(),
        driver: String::new(),
        pci_address: String::new(),
        vram_bytes: None,
        render_nodes: Vec::new(),
    };

    if let Ok(entries) = fs::read_dir(drm) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("card") || name.contains('-') {
                continue;
            }
            let device = entry.path().join("device");
            let uevent = read_path(&device.join("uevent"));
            let is_amd = uevent
                .as_deref()
                .is_some_and(|u| u.contains("DRIVER=amdgpu"));
            let vendor = read_path(&device.join("vendor"));
            if !is_amd && vendor.as_deref() != Some("0x1002") {
                continue;
            }

            spec.detected = true;
            spec.name = read_path(&device.join("product_name"))
                .or_else(|| read_path(&device.join("product_number")))
                .unwrap_or_else(|| "AMD GPU".into());
            spec.driver = uevent
                .as_deref()
                .and_then(|u| {
                    u.lines()
                        .find_map(|l| l.strip_prefix("DRIVER=").map(str::to_string))
                })
                .unwrap_or_else(|| "amdgpu".into());
            spec.pci_address = uevent
                .as_deref()
                .and_then(|u| {
                    u.lines()
                        .find_map(|l| l.strip_prefix("PCI_SLOT_NAME=").map(str::to_string))
                })
                .unwrap_or_default();
            spec.vram_bytes =
                read_path(&device.join("mem_info_vram_total")).and_then(|s| s.parse().ok());
            break;
        }
    }

    if let Ok(entries) = fs::read_dir("/dev/dri") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("renderD") {
                spec.render_nodes.push(format!("/dev/dri/{name}"));
            }
        }
    }

    spec
}

fn read_npu_spec() -> NpuSpec {
    let device = Path::new("/sys/class/accel/accel0/device");
    if !device.exists() {
        return NpuSpec {
            detected: false,
            name: "Not detected".into(),
            driver: String::new(),
            firmware: String::new(),
            pci_address: String::new(),
            power_state: String::new(),
            utilization_available: false,
        };
    }

    let uevent = read_path(&device.join("uevent")).unwrap_or_default();
    let driver = uevent
        .lines()
        .find_map(|l| l.strip_prefix("DRIVER=").map(str::to_string))
        .unwrap_or_else(|| "amdxdna".into());

    NpuSpec {
        detected: true,
        name: "AMD XDNA NPU".into(),
        driver,
        firmware: read_path(&device.join("fw_version")).unwrap_or_else(|| "unknown".into()),
        pci_address: uevent
            .lines()
            .find_map(|l| l.strip_prefix("PCI_SLOT_NAME=").map(str::to_string))
            .unwrap_or_default(),
        power_state: read_path(&device.join("power_state")).unwrap_or_else(|| "unknown".into()),
        utilization_available: device.join("utilization").exists(),
    }
}

fn read_memory_spec() -> MemorySpec {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total = 0u64;
    let mut swap = 0u64;
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_kb(line).unwrap_or(0) * 1024;
        } else if line.starts_with("SwapTotal:") {
            swap = parse_kb(line).unwrap_or(0) * 1024;
        }
    }
    MemorySpec {
        total_bytes: total,
        swap_bytes: swap,
        channels: "Unified system memory".into(),
    }
}

fn read_storage() -> Vec<StorageDevice> {
    let mut devices = Vec::new();
    let block = Path::new("/sys/block");
    let Ok(entries) = fs::read_dir(block) else {
        return devices;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("loop") || name.starts_with("ram") {
            continue;
        }
        let base = entry.path();
        let size = read_path(&base.join("size"))
            .and_then(|s| s.parse::<u64>().ok())
            .map(|sectors| sectors * 512)
            .unwrap_or(0);
        let model = read_path(&base.join("device/model")).unwrap_or_else(|| name.clone());
        let kind = if name.starts_with("nvme") {
            "NVMe SSD"
        } else if name.starts_with("sd") {
            "SATA/SCSI"
        } else {
            "Block device"
        };
        devices.push(StorageDevice {
            name,
            size_bytes: size,
            model: model.trim().to_string(),
            kind: kind.into(),
        });
    }
    devices
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

fn parse_kb(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1)?.parse().ok()
}

fn read_path(path: &Path) -> Option<String> {
    read_file(path.to_str()?)
}
fn read_file(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
