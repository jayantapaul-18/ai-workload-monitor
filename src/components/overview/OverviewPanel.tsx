import type { MetricsSnapshot } from "../../types/metrics";
import { formatBytes, formatBytesPerSec, formatUptime } from "../../utils/format";
import { GaugeRing } from "../shared/GaugeRing";
import { ProgressBar } from "../shared/ProgressBar";
import { Sparkline } from "../shared/Sparkline";
import { StatCard } from "../shared/StatCard";
import { ActiveAiWorkloads } from "./ActiveAiWorkloads";

interface OverviewPanelProps {
  metrics: MetricsSnapshot;
  history: number[];
  gpuHistory: number[];
  memHistory: number[];
  rxHistory: number[];
  txHistory: number[];
}

export function OverviewPanel({
  metrics,
  history,
  gpuHistory,
  memHistory,
  rxHistory,
  txHistory,
}: OverviewPanelProps) {
  const { cpu, memory, gpu, npu, processes, network, system, ai_runtimes } = metrics;

  return (
    <div className="panel overview-panel">
      <div className="panel__header">
        <h2>Performance Overview</h2>
        <p>Real-time snapshot of your AI workstation</p>
      </div>

      <div className="overview-gauges">
        <GaugeRing value={cpu.overall_usage} label="CPU" sublabel={`Load ${cpu.load_avg[0].toFixed(2)}`} />
        <GaugeRing
          value={memory.usage_percent}
          label="Memory"
          sublabel={`${formatBytes(memory.used_bytes)} used`}
        />
        <GaugeRing
          value={gpu.available ? gpu.usage_percent : 0}
          label="GPU"
          sublabel={gpu.available ? gpu.name : "Unavailable"}
        />
        <GaugeRing
          value={npu.usage_percent ?? 0}
          label="NPU"
          sublabel={
            npu.available
              ? npu.usage_percent != null
                ? npu.power_state
                : `Online · ${npu.power_state}`
              : "Unavailable"
          }
        />
      </div>

      <ActiveAiWorkloads aiRuntimes={ai_runtimes} />

      <div className="stat-grid">
        <StatCard
          title="AI Workloads"
          value={String(ai_runtimes.workloads.length || processes.ai_workloads.length)}
          subtitle={
            ai_runtimes.workloads.length > 0
              ? `${ai_runtimes.workloads.length} model(s) via runtime API`
              : "Process-level AI detection"
          }
          accent="var(--purple)"
        >
          <Sparkline data={history} color="var(--purple)" height={36} />
        </StatCard>
        <StatCard
          title="Network"
          value={formatBytesPerSec(network.total_rx_bytes_per_sec + network.total_tx_bytes_per_sec)}
          subtitle={`↓ ${formatBytesPerSec(network.total_rx_bytes_per_sec)} · ↑ ${formatBytesPerSec(network.total_tx_bytes_per_sec)}`}
          accent="var(--cyan)"
        >
          <div className="dual-sparkline">
            <Sparkline data={rxHistory} color="var(--cyan)" height={28} />
            <Sparkline data={txHistory} color="var(--warning)" height={28} />
          </div>
        </StatCard>
        <StatCard
          title="System Uptime"
          value={formatUptime(system.uptime_secs)}
          subtitle={`${system.hostname} · ${system.cpu_threads} threads`}
          accent="var(--accent)"
        />
        <StatCard
          title="GPU Temp"
          value={gpu.temperature_c ? `${gpu.temperature_c.toFixed(0)}°C` : "—"}
          subtitle={gpu.power_watts ? `${gpu.power_watts.toFixed(1)} W` : "Power n/a"}
          accent="var(--danger)"
        >
          <Sparkline data={gpuHistory} color="var(--danger)" height={36} max={100} />
        </StatCard>
      </div>

      <div className="overview-bars">
        <ProgressBar value={cpu.overall_usage} label="CPU Usage" />
        <ProgressBar value={memory.usage_percent} label="Memory Usage" />
        {gpu.available && <ProgressBar value={gpu.usage_percent} label="GPU Usage" />}
        <ProgressBar
          value={
            memory.swap_total_bytes
              ? (memory.swap_used_bytes / memory.swap_total_bytes) * 100
              : 0
          }
          label="Swap Usage"
        />
      </div>

      <div className="trend-row">
        <div className="trend-card">
          <span className="trend-card__label">CPU Trend</span>
          <Sparkline data={history} color="var(--accent)" height={56} max={100} />
        </div>
        <div className="trend-card">
          <span className="trend-card__label">Memory Trend</span>
          <Sparkline data={memHistory} color="var(--warning)" height={56} max={100} />
        </div>
      </div>
    </div>
  );
}
