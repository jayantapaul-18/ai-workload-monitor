import { useState } from "react";
import type { AiRuntimeMetrics, ProcessEntry, ProcessMetrics } from "../../types/metrics";
import { formatBytes } from "../../utils/format";
import { ActiveAiWorkloads } from "../overview/ActiveAiWorkloads";

interface ProcessPanelProps {
  processes: ProcessMetrics;
  aiRuntimes: AiRuntimeMetrics;
}

type ProcessTab = "ai" | "cpu" | "memory" | "background";

function ProcessTable({ rows }: { rows: ProcessEntry[] }) {
  if (rows.length === 0) {
    return <div className="empty-table">No processes in this category</div>;
  }

  return (
    <table className="process-table">
      <thead>
        <tr>
          <th>PID</th>
          <th>Name</th>
          <th>CPU</th>
          <th>Memory</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((proc) => (
          <tr key={proc.pid} className={proc.is_ai ? "process-row--ai" : undefined}>
            <td className="mono">{proc.pid}</td>
            <td>
              <div className="process-name">
                {proc.is_ai && <span className="ai-tag">AI</span>}
                {proc.name}
              </div>
              <div className="process-cmd">{proc.cmd}</div>
            </td>
            <td className="mono">{proc.cpu_percent.toFixed(1)}%</td>
            <td className="mono">{formatBytes(proc.memory_bytes)}</td>
            <td>{proc.status}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function ProcessPanel({ processes, aiRuntimes }: ProcessPanelProps) {
  const [tab, setTab] = useState<ProcessTab>("ai");

  const tabs: { id: ProcessTab; label: string; rows: ProcessEntry[] }[] = [
    { id: "ai", label: `AI Workloads (${processes.ai_workloads.length})`, rows: processes.ai_workloads },
    { id: "cpu", label: "Top CPU", rows: processes.top_cpu },
    { id: "memory", label: "Top Memory", rows: processes.top_memory },
    { id: "background", label: "Background", rows: processes.background },
  ];

  const active = tabs.find((t) => t.id === tab)!;

  return (
    <div className="panel process-panel">
      <div className="panel__header">
        <h2>Processes</h2>
        <p>AI-tagged workloads, resource hogs, and background services</p>
      </div>

      <div className="sub-tabs">
        {tabs.map((t) => (
          <button
            key={t.id}
            className={`sub-tab ${tab === t.id ? "sub-tab--active" : ""}`}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>

      {tab === "ai" && <ActiveAiWorkloads aiRuntimes={aiRuntimes} />}

      <ProcessTable rows={active.rows} />
    </div>
  );
}
