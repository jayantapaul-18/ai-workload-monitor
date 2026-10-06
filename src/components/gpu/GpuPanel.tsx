import type { GpuMetrics } from "../../types/metrics";
import { formatBytes } from "../../utils/format";
import { GaugeRing } from "../shared/GaugeRing";
import { ProgressBar } from "../shared/ProgressBar";
import { Sparkline } from "../shared/Sparkline";

interface GpuPanelProps {
  gpu: GpuMetrics;
  history: number[];
}

export function GpuPanel({ gpu, history }: GpuPanelProps) {
  if (!gpu.available) {
    return (
      <div className="panel">
        <div className="empty-state">
          <h2>No GPU Detected</h2>
          <p>AMD or NVIDIA GPU metrics will appear here when available.</p>
        </div>
      </div>
    );
  }

  const vramPercent =
    gpu.vram_used_bytes && gpu.vram_total_bytes
      ? (gpu.vram_used_bytes / gpu.vram_total_bytes) * 100
      : null;

  return (
    <div className="panel gpu-panel">
      <div className="panel__header">
        <h2>GPU — {gpu.name}</h2>
        <p>Integrated/discrete graphics compute monitor</p>
      </div>

      <div className="gpu-layout">
        <div className="gpu-main">
          <GaugeRing value={gpu.usage_percent} label="Utilization" size={160} />
          <Sparkline data={history} color="var(--danger)" height={72} max={100} />
        </div>

        <div className="gpu-stats">
          <div className="detail-row">
            <span>Temperature</span>
            <strong>{gpu.temperature_c?.toFixed(0) ?? "—"}°C</strong>
          </div>
          <div className="detail-row">
            <span>Power Draw</span>
            <strong>{gpu.power_watts?.toFixed(1) ?? "—"} W</strong>
          </div>
          <div className="detail-row">
            <span>Clock</span>
            <strong>{gpu.frequency_mhz?.toFixed(0) ?? "—"} MHz</strong>
          </div>
          {vramPercent !== null && (
            <>
              <div className="detail-row">
                <span>VRAM Used</span>
                <strong>
                  {formatBytes(gpu.vram_used_bytes!)} / {formatBytes(gpu.vram_total_bytes!)}
                </strong>
              </div>
              <ProgressBar value={vramPercent} label="VRAM" />
            </>
          )}
          <ProgressBar value={gpu.usage_percent} label="GPU Load" />
        </div>
      </div>
    </div>
  );
}
