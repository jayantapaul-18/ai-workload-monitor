import type { CpuMetrics } from "../../types/metrics";
import { CoreBox } from "./CoreBox";
import { ProgressBar } from "../shared/ProgressBar";
import { Sparkline } from "../shared/Sparkline";

interface CpuPanelProps {
  cpu: CpuMetrics;
  history: number[];
}

export function CpuPanel({ cpu, history }: CpuPanelProps) {
  return (
    <div className="panel cpu-panel">
      <div className="panel__header">
        <h2>CPU Cores</h2>
        <p>
          {cpu.cores.length} logical cores · Overall {cpu.overall_usage.toFixed(1)}% ·
          Load {cpu.load_avg.map((v) => v.toFixed(2)).join(" / ")}
          {cpu.frequency_mhz ? ` · ${cpu.frequency_mhz.toFixed(0)} MHz` : ""}
        </p>
      </div>

      <div className="cpu-summary">
        <ProgressBar value={cpu.overall_usage} label="Aggregate CPU" />
        <div className="trend-card trend-card--inline">
          <span className="trend-card__label">60s History</span>
          <Sparkline data={history} color="var(--accent)" height={64} max={100} />
        </div>
      </div>

      <div className="core-grid">
        {cpu.cores.map((core) => (
          <CoreBox key={core.id} core={core} />
        ))}
      </div>

      <div className="load-avg-row">
        {(["1 min", "5 min", "15 min"] as const).map((label, i) => (
          <div key={label} className="load-avg-card">
            <span className="load-avg-card__label">{label}</span>
            <span className="load-avg-card__value">{cpu.load_avg[i].toFixed(2)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
