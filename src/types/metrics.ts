export interface CoreMetrics {
  id: number;
  usage: number;
  frequency_mhz: number | null;
}

export interface CpuMetrics {
  cores: CoreMetrics[];
  overall_usage: number;
  load_avg: [number, number, number];
  frequency_mhz: number | null;
}

export interface MemoryMetrics {
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  cached_bytes: number;
  swap_total_bytes: number;
  swap_used_bytes: number;
  usage_percent: number;
}

export interface GpuMetrics {
  available: boolean;
  name: string;
  usage_percent: number;
  temperature_c: number | null;
  power_watts: number | null;
  frequency_mhz: number | null;
  vram_used_bytes: number | null;
  vram_total_bytes: number | null;
}

export interface NpuMetrics {
  available: boolean;
  name: string;
  driver: string;
  firmware_version: string;
  power_state: string;
  usage_percent: number | null;
}

export interface ProcessEntry {
  pid: number;
  name: string;
  cmd: string;
  cpu_percent: number;
  memory_bytes: number;
  status: string;
  is_ai: boolean;
}

export interface ProcessMetrics {
  top_cpu: ProcessEntry[];
  top_memory: ProcessEntry[];
  background: ProcessEntry[];
  ai_workloads: ProcessEntry[];
}

export interface InterfaceMetrics {
  name: string;
  kind: string;
  mac: string;
  state: string;
  mtu: number;
  ipv4: string[];
  rx_bytes_per_sec: number;
  tx_bytes_per_sec: number;
  rx_total_bytes: number;
  tx_total_bytes: number;
  speed_mbps: number | null;
}

export interface WifiMetrics {
  interface: string;
  ssid: string;
  signal_dbm: number | null;
  signal_percent: number | null;
  frequency_mhz: number | null;
  bitrate_mbps: number | null;
  connected: boolean;
}

export interface NetworkMetrics {
  interfaces: InterfaceMetrics[];
  wifi: WifiMetrics | null;
  total_rx_bytes_per_sec: number;
  total_tx_bytes_per_sec: number;
}

export interface SystemInfo {
  hostname: string;
  os: string;
  kernel: string;
  cpu_model: string;
  cpu_cores: number;
  cpu_threads: number;
  total_memory_bytes: number;
  uptime_secs: number;
  architecture: string;
}

export interface AiWorkloadEntry {
  runtime: string;
  model: string;
  status: string;
  accelerator: string;
  vram_bytes: number | null;
  memory_on_gpu: boolean;
  tokens_per_sec: number | null;
  throughput_kind: string;
  parameter_size: string | null;
  quantization: string | null;
  context_length: number | null;
  requests_running: number | null;
}

export interface BottleneckInsight {
  kind: string;
  severity: string;
  message: string;
  suggestion: string;
}

export interface AiRuntimeMetrics {
  workloads: AiWorkloadEntry[];
  bottleneck: BottleneckInsight | null;
  runtimes_online: string[];
}

export interface MetricsSnapshot {
  timestamp: number;
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  gpu: GpuMetrics;
  npu: NpuMetrics;
  processes: ProcessMetrics;
  network: NetworkMetrics;
  system: SystemInfo;
  ai_runtimes: AiRuntimeMetrics;
}

export type TabId =
  | "overview"
  | "cpu"
  | "gpu"
  | "npu"
  | "processes"
  | "network"
  | "hardware"
  | "architecture"
  | "settings";

export const EMPTY_METRICS: MetricsSnapshot = {
  timestamp: 0,
  cpu: { cores: [], overall_usage: 0, load_avg: [0, 0, 0], frequency_mhz: null },
  memory: {
    total_bytes: 0,
    used_bytes: 0,
    available_bytes: 0,
    cached_bytes: 0,
    swap_total_bytes: 0,
    swap_used_bytes: 0,
    usage_percent: 0,
  },
  gpu: {
    available: false,
    name: "—",
    usage_percent: 0,
    temperature_c: null,
    power_watts: null,
    frequency_mhz: null,
    vram_used_bytes: null,
    vram_total_bytes: null,
  },
  npu: {
    available: false,
    name: "—",
    driver: "",
    firmware_version: "",
    power_state: "",
    usage_percent: null,
  },
  processes: { top_cpu: [], top_memory: [], background: [], ai_workloads: [] },
  network: { interfaces: [], wifi: null, total_rx_bytes_per_sec: 0, total_tx_bytes_per_sec: 0 },
  system: {
    hostname: "—",
    os: "—",
    kernel: "—",
    cpu_model: "—",
    cpu_cores: 0,
    cpu_threads: 0,
    total_memory_bytes: 0,
    uptime_secs: 0,
    architecture: "—",
  },
  ai_runtimes: { workloads: [], bottleneck: null, runtimes_online: [] },
};
