export interface PlatformSpec {
  hostname: string;
  os: string;
  kernel: string;
  architecture: string;
  desktop: string;
}

export interface CpuSpec {
  model: string;
  vendor: string;
  cores: number;
  threads: number;
  max_frequency_mhz: number | null;
  cache_l3_kb: number | null;
  flags: string[];
  virtualization: string;
}

export interface GpuSpec {
  detected: boolean;
  name: string;
  driver: string;
  pci_address: string;
  vram_bytes: number | null;
  render_nodes: string[];
}

export interface NpuSpec {
  detected: boolean;
  name: string;
  driver: string;
  firmware: string;
  pci_address: string;
  power_state: string;
  utilization_available: boolean;
}

export interface MemorySpec {
  total_bytes: number;
  swap_bytes: number;
  channels: string;
}

export interface StorageDevice {
  name: string;
  size_bytes: number;
  model: string;
  kind: string;
}

export interface AppSpec {
  name: string;
  version: string;
  identifier: string;
}

export interface HardwareProfile {
  platform: PlatformSpec;
  cpu: CpuSpec;
  gpu: GpuSpec;
  npu: NpuSpec;
  memory: MemorySpec;
  storage: StorageDevice[];
  app: AppSpec;
}
