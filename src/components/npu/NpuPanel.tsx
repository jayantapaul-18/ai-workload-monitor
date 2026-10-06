import type { NpuMetrics } from "../../types/metrics";
import { StatCard } from "../shared/StatCard";

interface NpuPanelProps {
  npu: NpuMetrics;
}

export function NpuPanel({ npu }: NpuPanelProps) {
  if (!npu.available) {
    return (
      <div className="panel">
        <div className="empty-state">
          <h2>No NPU Detected</h2>
          <p>AMD XDNA or other AI accelerator metrics will appear here.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="panel npu-panel">
      <div className="panel__header">
        <h2>{npu.name}</h2>
        <p>Neural Processing Unit — AI inference accelerator</p>
      </div>

      <div className="npu-status">
        <div className="npu-badge npu-badge--active">Online</div>
        <span className="npu-power">{npu.power_state}</span>
      </div>

      <div className="stat-grid">
        <StatCard title="Driver" value={npu.driver} accent="var(--purple)" />
        <StatCard title="Firmware" value={npu.firmware_version} accent="var(--cyan)" />
        <StatCard title="Power State" value={npu.power_state} accent="var(--accent)" />
        <StatCard
          title="Utilization"
          value={npu.usage_percent != null ? `${npu.usage_percent.toFixed(0)}%` : "Pending"}
          subtitle="Kernel metrics not yet exposed"
          accent="var(--warning)"
        />
      </div>

      <div className="info-banner">
        AMD XDNA utilization will populate automatically when the Linux kernel or XRT
        exposes runtime counters. Device health and power state are monitored now.
      </div>
    </div>
  );
}
