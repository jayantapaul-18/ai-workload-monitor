use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub gpu: GpuMetrics,
    pub npu: NpuMetrics,
    pub processes: ProcessMetrics,
    pub network: NetworkMetrics,
    pub system: SystemInfo,
    pub ai_runtimes: AiRuntimeMetrics,
}

impl MetricsSnapshot {
    pub fn empty() -> Self {
        Self {
            timestamp: 0,
            cpu: CpuMetrics {
                cores: Vec::new(),
                overall_usage: 0.0,
                load_avg: [0.0, 0.0, 0.0],
                frequency_mhz: None,
            },
            memory: MemoryMetrics {
                total_bytes: 0,
                used_bytes: 0,
                available_bytes: 0,
                cached_bytes: 0,
                swap_total_bytes: 0,
                swap_used_bytes: 0,
                usage_percent: 0.0,
            },
            gpu: GpuMetrics {
                available: false,
                name: String::new(),
                usage_percent: 0.0,
                temperature_c: None,
                power_watts: None,
                frequency_mhz: None,
                vram_used_bytes: None,
                vram_total_bytes: None,
            },
            npu: NpuMetrics {
                available: false,
                name: String::new(),
                driver: String::new(),
                firmware_version: String::new(),
                power_state: String::new(),
                usage_percent: None,
            },
            processes: ProcessMetrics {
                top_cpu: Vec::new(),
                top_memory: Vec::new(),
                background: Vec::new(),
                ai_workloads: Vec::new(),
            },
            network: NetworkMetrics {
                interfaces: Vec::new(),
                wifi: None,
                total_rx_bytes_per_sec: 0,
                total_tx_bytes_per_sec: 0,
            },
            system: SystemInfo {
                hostname: String::new(),
                os: String::new(),
                kernel: String::new(),
                cpu_model: String::new(),
                cpu_cores: 0,
                cpu_threads: 0,
                total_memory_bytes: 0,
                uptime_secs: 0,
                architecture: String::new(),
            },
            ai_runtimes: AiRuntimeMetrics::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub cores: Vec<CoreMetrics>,
    pub overall_usage: f32,
    pub load_avg: [f32; 3],
    pub frequency_mhz: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMetrics {
    pub id: u32,
    pub usage: f32,
    pub frequency_mhz: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub available: bool,
    pub name: String,
    pub usage_percent: f32,
    pub temperature_c: Option<f32>,
    pub power_watts: Option<f32>,
    pub frequency_mhz: Option<f32>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuMetrics {
    pub available: bool,
    pub name: String,
    pub driver: String,
    pub firmware_version: String,
    pub power_state: String,
    pub usage_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub top_cpu: Vec<ProcessEntry>,
    pub top_memory: Vec<ProcessEntry>,
    pub background: Vec<ProcessEntry>,
    pub ai_workloads: Vec<ProcessEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub is_ai: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<InterfaceMetrics>,
    pub wifi: Option<WifiMetrics>,
    pub total_rx_bytes_per_sec: u64,
    pub total_tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMetrics {
    pub name: String,
    pub kind: String,
    pub mac: String,
    pub state: String,
    pub mtu: u32,
    pub ipv4: Vec<String>,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub rx_total_bytes: u64,
    pub tx_total_bytes: u64,
    pub speed_mbps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiMetrics {
    pub interface: String,
    pub ssid: String,
    pub signal_dbm: Option<i32>,
    pub signal_percent: Option<u32>,
    pub frequency_mhz: Option<u32>,
    pub bitrate_mbps: Option<u32>,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub total_memory_bytes: u64,
    pub uptime_secs: u64,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiRuntimeMetrics {
    pub workloads: Vec<AiWorkloadEntry>,
    pub bottleneck: Option<BottleneckInsight>,
    pub runtimes_online: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiWorkloadEntry {
    pub runtime: String,
    pub model: String,
    pub status: String,
    pub accelerator: String,
    pub vram_bytes: Option<u64>,
    pub memory_on_gpu: bool,
    pub tokens_per_sec: Option<f32>,
    pub throughput_kind: String,
    pub parameter_size: Option<String>,
    pub quantization: Option<String>,
    pub context_length: Option<u32>,
    pub requests_running: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckInsight {
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub suggestion: String,
}
