import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import type { AppSettings } from "../../types/settings";
import type { SettingsSection } from "../../types/settings";
import { ACCENT_OPTIONS, POLL_OPTIONS, THEME_OPTIONS } from "../../types/settings";
import { Select, SettingRow, SettingSection, Slider, Toggle } from "./SettingControls";

const NAV: { id: SettingsSection; label: string; icon: string }[] = [
  { id: "general", label: "General", icon: "⚙" },
  { id: "monitoring", label: "Monitoring", icon: "◉" },
  { id: "alerts", label: "Alerts", icon: "⚠" },
  { id: "ai", label: "AI Workloads", icon: "⬡" },
  { id: "appearance", label: "Appearance", icon: "◐" },
  { id: "advanced", label: "Advanced", icon: "⌘" },
  { id: "about", label: "About", icon: "ℹ" },
];

interface SettingsPanelProps {
  settings: AppSettings;
  settingsPath: string;
  saving: boolean;
  onUpdate: (patch: Partial<AppSettings>) => void;
  onReset: () => void;
}

export function SettingsPanel({
  settings,
  settingsPath,
  saving,
  onUpdate,
  onReset,
}: SettingsPanelProps) {
  const [section, setSection] = useState<SettingsSection>("general");
  const [keywordsText, setKeywordsText] = useState(settings.aiKeywords.join(", "));
  const [exportStatus, setExportStatus] = useState("");
  const [alerts, setAlerts] = useState<string[]>([]);

  useEffect(() => {
    setKeywordsText(settings.aiKeywords.join(", "));
  }, [settings.aiKeywords]);

  useEffect(() => {
    const unlisten = listen<{ title: string; message: string }>("alert-triggered", (e) => {
      setAlerts((prev) => [`${e.payload.title}: ${e.payload.message}`, ...prev].slice(0, 5));
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const saveKeywords = () => {
    const keywords = keywordsText
      .split(/[,\n]/)
      .map((k) => k.trim().toLowerCase())
      .filter(Boolean);
    onUpdate({ aiKeywords: keywords });
  };

  const exportSnapshot = async () => {
    try {
      const json = await invoke<string>("export_metrics_snapshot");
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `metrics-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
      setExportStatus("Exported successfully");
    } catch {
      setExportStatus("Export failed — wait for first metrics tick");
    }
  };

  return (
    <div className="settings-panel">
      <aside className="settings-nav">
        <div className="settings-nav__header">
          <h2>Settings</h2>
          <p>Configure your monitor</p>
        </div>
        {NAV.map((item) => (
          <button
            key={item.id}
            className={`settings-nav__item ${section === item.id ? "settings-nav__item--active" : ""}`}
            onClick={() => setSection(item.id)}
          >
            <span>{item.icon}</span>
            {item.label}
          </button>
        ))}
        {saving && <div className="settings-saving">Saving…</div>}
      </aside>

      <div className="settings-content">
        {section === "general" && (
          <>
            <SettingSection title="General" description="Application behavior and startup">
              <SettingRow label="Start minimized to tray" description="Launch hidden — you'll get a tray notification. Click tray icon or right-click → Show Dashboard.">
                <Toggle checked={settings.startMinimized} onChange={(v) => onUpdate({ startMinimized: v })} />
              </SettingRow>
              <SettingRow label="Close to tray" description="Keep running when window is closed">
                <Toggle checked={settings.closeToTray} onChange={(v) => onUpdate({ closeToTray: v })} />
              </SettingRow>
              <SettingRow label="Show tray tooltip" description="Live metrics in tray hover text">
                <Toggle checked={settings.showTrayTooltip} onChange={(v) => onUpdate({ showTrayTooltip: v })} />
              </SettingRow>
              <SettingRow label="Tray popup on click" description="Compact metrics panel when clicking tray icon">
                <Toggle checked={settings.trayPopupEnabled} onChange={(v) => onUpdate({ trayPopupEnabled: v })} />
              </SettingRow>
              <SettingRow label="Launch at login" description="Start automatically on boot (requires desktop entry)">
                <Toggle checked={settings.launchAtLogin} onChange={(v) => onUpdate({ launchAtLogin: v })} />
              </SettingRow>
            </SettingSection>
          </>
        )}

        {section === "monitoring" && (
          <SettingSection title="Monitoring" description="Polling rates and data collection">
            <SettingRow label="Poll interval" description="How often metrics are collected">
              <Select
                value={settings.pollIntervalMs}
                options={POLL_OPTIONS}
                onChange={(v) => onUpdate({ pollIntervalMs: Number(v) })}
              />
            </SettingRow>
            <SettingRow label="Burst poll when focused" description="2× faster updates when dashboard is active">
              <Toggle
                checked={settings.burstPollWhenFocused}
                onChange={(v) => onUpdate({ burstPollWhenFocused: v })}
              />
            </SettingRow>
            <SettingRow label="Low power mode" description="Minimum 2s polling, disables burst">
              <Toggle checked={settings.lowPowerMode} onChange={(v) => onUpdate({ lowPowerMode: v })} />
            </SettingRow>
            <SettingRow label="Process list limit" description="Max processes per category">
              <Slider
                value={settings.processLimit}
                min={5}
                max={30}
                onChange={(v) => onUpdate({ processLimit: v })}
              />
            </SettingRow>
            <SettingRow label="History length" description="Sparkline data points (seconds)">
              <Slider
                value={settings.historyLength}
                min={30}
                max={300}
                step={10}
                onChange={(v) => onUpdate({ historyLength: v })}
              />
            </SettingRow>
          </SettingSection>
        )}

        {section === "alerts" && (
          <SettingSection title="Alerts" description="Threshold notifications for AI workloads">
            <SettingRow label="Enable alerts" description="OS desktop notifications + in-app toasts when thresholds are breached">
              <Toggle checked={settings.alertsEnabled} onChange={(v) => onUpdate({ alertsEnabled: v })} />
            </SettingRow>
            <div className="setting-actions">
              <button
                type="button"
                className="btn-secondary"
                onClick={() => invoke("test_notification").catch(() => {})}
              >
                Test notification
              </button>
            </div>
            <SettingRow label="CPU threshold" description="Alert when overall CPU exceeds">
              <Slider
                value={settings.cpuAlertThreshold}
                min={50}
                max={100}
                suffix="%"
                onChange={(v) => onUpdate({ cpuAlertThreshold: v })}
              />
            </SettingRow>
            <SettingRow label="Memory threshold" description="Alert when RAM usage exceeds">
              <Slider
                value={settings.memoryAlertThreshold}
                min={50}
                max={100}
                suffix="%"
                onChange={(v) => onUpdate({ memoryAlertThreshold: v })}
              />
            </SettingRow>
            <SettingRow label="GPU temperature" description="Alert when GPU temp exceeds">
              <Slider
                value={settings.gpuTempAlertThreshold}
                min={60}
                max={100}
                suffix="°C"
                onChange={(v) => onUpdate({ gpuTempAlertThreshold: v })}
              />
            </SettingRow>
            <SettingRow label="Alert cooldown" description="Minimum seconds between same alert">
              <Slider
                value={settings.alertCooldownSecs}
                min={15}
                max={300}
                step={15}
                suffix="s"
                onChange={(v) => onUpdate({ alertCooldownSecs: v })}
              />
            </SettingRow>
            <SettingRow label="AI process changes" description="Notify when AI workload count changes">
              <Toggle
                checked={settings.notifyAiProcessChanges}
                onChange={(v) => onUpdate({ notifyAiProcessChanges: v })}
              />
            </SettingRow>
            {alerts.length > 0 && (
              <div className="alert-log">
                <h4>Recent alerts</h4>
                {alerts.map((a, i) => (
                  <div key={i} className="alert-log__item">{a}</div>
                ))}
              </div>
            )}
          </SettingSection>
        )}

        {section === "ai" && (
          <SettingSection title="AI Workloads" description="Runtime connectors and process detection">
            <SettingRow
              label="AI runtime intelligence"
              description="Query Ollama and vLLM for loaded models, throughput, and bottlenecks"
            >
              <Toggle
                checked={settings.aiIntelligenceEnabled}
                onChange={(v) => onUpdate({ aiIntelligenceEnabled: v })}
              />
            </SettingRow>
            <SettingRow label="Ollama URL" description="Base URL for Ollama API (default port 11434)">
              <input
                className="setting-input"
                type="text"
                value={settings.ollamaUrl}
                onChange={(e) => onUpdate({ ollamaUrl: e.target.value })}
                disabled={!settings.aiIntelligenceEnabled}
              />
            </SettingRow>
            <SettingRow label="vLLM metrics URL" description="Prometheus /metrics endpoint for vLLM">
              <input
                className="setting-input"
                type="text"
                value={settings.vllmMetricsUrl}
                onChange={(e) => onUpdate({ vllmMetricsUrl: e.target.value })}
                disabled={!settings.aiIntelligenceEnabled}
              />
            </SettingRow>
            <SettingRow label="Highlight AI processes" description="Mark AI-tagged rows in process list">
              <Toggle
                checked={settings.highlightAiInProcesses}
                onChange={(v) => onUpdate({ highlightAiInProcesses: v })}
              />
            </SettingRow>
            <SettingRow label="Auto-focus AI tab" description="Switch to AI workloads when new process detected">
              <Toggle checked={settings.autoFocusAiTab} onChange={(v) => onUpdate({ autoFocusAiTab: v })} />
            </SettingRow>
            <div className="setting-keywords">
              <label>Detection keywords (comma-separated)</label>
              <textarea
                value={keywordsText}
                onChange={(e) => setKeywordsText(e.target.value)}
                rows={6}
                placeholder="python, ollama, vllm, docker…"
              />
              <button type="button" className="btn-secondary" onClick={saveKeywords}>
                Apply keywords
              </button>
            </div>
          </SettingSection>
        )}

        {section === "appearance" && (
          <SettingSection title="Appearance" description="Visual theme and layout">
            <SettingRow label="Color theme" description="Dark or light interface">
              <div className="theme-picker">
                {THEME_OPTIONS.map((opt) => (
                  <button
                    key={opt.value}
                    type="button"
                    className={`theme-btn ${settings.colorTheme === opt.value ? "theme-btn--active" : ""}`}
                    onClick={() => onUpdate({ colorTheme: opt.value })}
                  >
                    <span>{opt.icon}</span>
                    {opt.label}
                  </button>
                ))}
              </div>
            </SettingRow>
            <SettingRow label="Compact sidebar" description="Icon-only left navigation rail">
              <Toggle
                checked={settings.compactSidebar}
                onChange={(v) => onUpdate({ compactSidebar: v })}
              />
            </SettingRow>
            <SettingRow label="Accent color" description="Primary UI accent">
              <div className="accent-picker">
                {ACCENT_OPTIONS.map((opt) => (
                  <button
                    key={opt.value}
                    type="button"
                    className={`accent-swatch ${settings.accentTheme === opt.value ? "accent-swatch--active" : ""}`}
                    style={{ "--swatch": opt.color } as React.CSSProperties}
                    onClick={() => onUpdate({ accentTheme: opt.value })}
                    title={opt.label}
                  />
                ))}
              </div>
            </SettingRow>
            <SettingRow label="Compact density" description="Tighter spacing in panels">
              <Toggle checked={settings.compactDensity} onChange={(v) => onUpdate({ compactDensity: v })} />
            </SettingRow>
            <SettingRow label="Show sparklines" description="Trend charts in overview">
              <Toggle checked={settings.showSparklines} onChange={(v) => onUpdate({ showSparklines: v })} />
            </SettingRow>
            <SettingRow label="Animate gauges" description="Smooth gauge transitions">
              <Toggle checked={settings.animateGauges} onChange={(v) => onUpdate({ animateGauges: v })} />
            </SettingRow>
          </SettingSection>
        )}

        {section === "advanced" && (
          <SettingSection title="Advanced" description="Export and power-user options">
            <SettingRow label="Include processes in export" description="Full process list in JSON export">
              <Toggle
                checked={settings.exportIncludeProcesses}
                onChange={(v) => onUpdate({ exportIncludeProcesses: v })}
              />
            </SettingRow>
            <div className="setting-actions">
              <button type="button" className="btn-primary" onClick={exportSnapshot}>
                Export current metrics (JSON)
              </button>
              {exportStatus && <span className="muted">{exportStatus}</span>}
            </div>
            <div className="setting-actions">
              <button type="button" className="btn-danger" onClick={onReset}>
                Reset all settings to defaults
              </button>
            </div>
          </SettingSection>
        )}

        {section === "about" && (
          <SettingSection title="About" description="AI Workload Monitor">
            <div className="about-grid">
              <div><span>Version</span><strong>0.1.0</strong></div>
              <div><span>Stack</span><strong>Tauri 2 + Rust + React</strong></div>
              <div><span>License</span><strong>MIT (open source)</strong></div>
              <div><span>Settings file</span><code>{settingsPath || "~/.config/ai-workload-monitor/settings.json"}</code></div>
            </div>
            <div className="about-features">
              <h4>Feature tracker</h4>
              <p className="about-features__intro">
                Full list of shipped and planned features:{" "}
                <code>docs/FEATURES.md</code> in the project repo.
              </p>
              <h4>Next up (planned)</h4>
              <ul>
                <li>Compact tray popup with live mini-charts</li>
                <li>ComfyUI queue connector</li>
                <li>NPU utilization via XRT/amdxdna</li>
                <li>Historical CSV logging for training runs</li>
                <li>Per-GPU process VRAM attribution</li>
                <li>Docker container grouping</li>
                <li>Prometheus metrics export</li>
              </ul>
            </div>
          </SettingSection>
        )}
      </div>
    </div>
  );
}
