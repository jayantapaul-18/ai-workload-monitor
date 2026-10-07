import { useMemo, useState } from "react";
import type { AiRuntimeMetrics, ProcessEntry, ProcessMetrics } from "../../types/metrics";
import { formatBytes } from "../../utils/format";
import { ActiveAiWorkloads } from "../overview/ActiveAiWorkloads";

interface ProcessPanelProps {
  processes: ProcessMetrics;
  aiRuntimes: AiRuntimeMetrics;
}

type ProcessTab = "ai" | "cpu" | "memory" | "background";
type SortColumn = "pid" | "name" | "cpu" | "memory";
type SortDirection = "asc" | "desc";

interface ProcessTableProps {
  rows: ProcessEntry[];
  sortCol: SortColumn;
  sortDir: SortDirection;
  onSort: (col: SortColumn) => void;
  search: string;
}

function ProcessTable({ rows, sortCol, sortDir, onSort, search }: ProcessTableProps) {
  if (rows.length === 0) {
    return (
      <div className="empty-table">
        {search ? `No processes match "${search}"` : "No processes in this category"}
      </div>
    );
  }

  const renderSortIndicator = (col: SortColumn) => {
    if (sortCol !== col) return null;
    return <span className="sort-arrow">{sortDir === "asc" ? "▲" : "▼"}</span>;
  };

  return (
    <table className="process-table">
      <thead>
        <tr>
          <th className="sortable" onClick={() => onSort("pid")}>
            PID {renderSortIndicator("pid")}
          </th>
          <th className="sortable" onClick={() => onSort("name")}>
            Name {renderSortIndicator("name")}
          </th>
          <th className="sortable" onClick={() => onSort("cpu")}>
            CPU {renderSortIndicator("cpu")}
          </th>
          <th className="sortable" onClick={() => onSort("memory")}>
            Memory {renderSortIndicator("memory")}
          </th>
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
  const [search, setSearch] = useState("");
  const [sortCol, setSortCol] = useState<SortColumn>("cpu");
  const [sortDir, setSortDir] = useState<SortDirection>("desc");

  const tabs: { id: ProcessTab; label: string; rows: ProcessEntry[] }[] = [
    { id: "ai", label: `AI Workloads (${processes.ai_workloads.length})`, rows: processes.ai_workloads },
    { id: "cpu", label: "Top CPU", rows: processes.top_cpu },
    { id: "memory", label: "Top Memory", rows: processes.top_memory },
    { id: "background", label: "Background", rows: processes.background },
  ];

  const active = tabs.find((t) => t.id === tab)!;

  const handleSort = (col: SortColumn) => {
    if (sortCol === col) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortCol(col);
      setSortDir(col === "pid" || col === "name" ? "asc" : "desc");
    }
  };

  const processedRows = useMemo(() => {
    let list = active.rows;
    if (search.trim()) {
      const q = search.trim().toLowerCase();
      list = list.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.cmd.toLowerCase().includes(q) ||
          String(p.pid).includes(q)
      );
    }

    return [...list].sort((a, b) => {
      let cmp = 0;
      if (sortCol === "pid") cmp = a.pid - b.pid;
      else if (sortCol === "name") cmp = a.name.localeCompare(b.name);
      else if (sortCol === "cpu") cmp = a.cpu_percent - b.cpu_percent;
      else if (sortCol === "memory") cmp = a.memory_bytes - b.memory_bytes;
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [active.rows, search, sortCol, sortDir]);

  return (
    <div className="panel process-panel">
      <div className="panel__header">
        <h2>Processes</h2>
        <p>AI-tagged workloads, resource hogs, and background services</p>
      </div>

      <div className="process-toolbar">
        <div className="sub-tabs">
          {tabs.map((t) => (
            <button
              key={t.id}
              className={`sub-tab ${tab === t.id ? "sub-tab--active" : ""}`}
              onClick={() => {
                setTab(t.id);
                if (t.id === "memory") {
                  setSortCol("memory");
                  setSortDir("desc");
                } else if (t.id === "cpu") {
                  setSortCol("cpu");
                  setSortDir("desc");
                }
              }}
            >
              {t.label}
            </button>
          ))}
        </div>

        <div className="process-search">
          <input
            type="text"
            className="process-search__input"
            placeholder="Filter processes..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          {search && (
            <button
              type="button"
              className="process-search__clear"
              onClick={() => setSearch("")}
              title="Clear search"
            >
              ✕
            </button>
          )}
        </div>
      </div>

      {tab === "ai" && <ActiveAiWorkloads aiRuntimes={aiRuntimes} />}

      <ProcessTable
        rows={processedRows}
        sortCol={sortCol}
        sortDir={sortDir}
        onSort={handleSort}
        search={search}
      />
    </div>
  );
}
