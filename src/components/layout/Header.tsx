import type { MetricsSnapshot } from "../../types/metrics";
import { formatBytes, formatPercent } from "../../utils/format";

interface HeaderProps {
  metrics: MetricsSnapshot | null;
  connected: boolean;
}

const STAT_ITEMS = [
  { key: "cpu" as const, label: "CPU" },
  { key: "ram" as const, label: "RAM" },
  { key: "gpu" as const, label: "GPU" },
  { key: "ai" as const, label: "AI" },
];

function statValue(key: (typeof STAT_ITEMS)[number]["key"], metrics: MetricsSnapshot): string {
  switch (key) {
    case "cpu":
      return formatPercent(metrics.cpu.overall_usage);
    case "ram":
      return formatBytes(metrics.memory.used_bytes);
    case "gpu":
      return metrics.gpu.available ? formatPercent(metrics.gpu.usage_percent) : "—";
    case "ai":
      return String(metrics.processes.ai_workloads.length);
  }
}

export function Header({ metrics, connected }: HeaderProps) {
  return (
    <header className="header">
      <span className={`status-dot ${connected ? "status-dot--live" : ""}`}>
        {connected ? "Live" : "Connecting…"}
      </span>

      {metrics && (
        <div className="header__stats" role="group" aria-label="Live system metrics">
          {STAT_ITEMS.map((item) => (
            <div key={item.key} className="header-stat">
              <span className="header-stat__label">{item.label}</span>
              <strong className="header-stat__value">{statValue(item.key, metrics)}</strong>
            </div>
          ))}
        </div>
      )}
    </header>
  );
}
