import type { TabId } from "../../types/metrics";

const TABS: { id: TabId; label: string; icon: string }[] = [
  { id: "overview", label: "Overview", icon: "◉" },
  { id: "cpu", label: "CPU", icon: "▦" },
  { id: "gpu", label: "GPU", icon: "◈" },
  { id: "npu", label: "NPU", icon: "⬡" },
  { id: "processes", label: "Processes", icon: "☰" },
  { id: "network", label: "Network", icon: "⇄" },
  { id: "hardware", label: "Hardware", icon: "⌂" },
  { id: "architecture", label: "Architecture", icon: "⎇" },
];

interface SidebarProps {
  active: TabId;
  onChange: (tab: TabId) => void;
  aiCount: number;
  compact?: boolean;
}

export function Sidebar({ active, onChange, aiCount, compact = false }: SidebarProps) {
  return (
    <aside className={`sidebar ${compact ? "sidebar--compact" : ""}`}>
      <div className="sidebar__brand">
        <img
          src="/logo-64.png"
          alt=""
          className="sidebar__logo"
          width={compact ? 32 : 40}
          height={compact ? 32 : 40}
        />
        {!compact && (
          <div>
            <div className="sidebar__title">AI Workload</div>
            <div className="sidebar__subtitle">Monitor</div>
          </div>
        )}
      </div>

      <nav className="sidebar__nav">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={`nav-item ${active === tab.id ? "nav-item--active" : ""}`}
            onClick={() => onChange(tab.id)}
            title={compact ? tab.label : undefined}
          >
            <span className="nav-item__icon">{tab.icon}</span>
            <span className="nav-item__label">{tab.label}</span>
            {tab.id === "processes" && aiCount > 0 && (
              <span className="nav-item__badge">{aiCount}</span>
            )}
          </button>
        ))}
      </nav>

      <div className="sidebar__bottom">
        <button
          className={`nav-item nav-item--settings ${active === "settings" ? "nav-item--active" : ""}`}
          onClick={() => onChange("settings")}
          title={compact ? "Settings" : undefined}
        >
          <span className="nav-item__icon">⚙</span>
          <span className="nav-item__label">Settings</span>
        </button>
        {!compact && (
          <div className="sidebar__footer">
            <span>Local · Low overhead</span>
          </div>
        )}
      </div>
    </aside>
  );
}
