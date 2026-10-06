import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useMetrics } from "./hooks/useMetrics";
import { applyTheme } from "./hooks/useSettings";
import type { AppSettings } from "./types/settings";
import { formatPercent } from "./utils/format";
import "./App.css";
import "./tray-popup.css";

function MiniBar({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="tray-bar">
      <div className="tray-bar__head">
        <span>{label}</span>
        <strong>{formatPercent(value)}</strong>
      </div>
      <div className="tray-bar__track">
        <div className="tray-bar__fill" style={{ width: `${Math.min(value, 100)}%`, background: color }} />
      </div>
    </div>
  );
}

export function TrayPopupApp() {
  const { metrics, connected } = useMetrics();

  useEffect(() => {
    document.documentElement.dataset.view = "tray";
    invoke<AppSettings>("get_settings").then(applyTheme).catch(() => {});
  }, []);

  useEffect(() => {
    const popup = getCurrentWindow();
    const unlistenBlur = popup.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        popup.hide().catch(() => {});
      }
    });

    const unlistenMetrics = listen("metrics-update", () => {});

    return () => {
      unlistenBlur.then((fn) => fn());
      unlistenMetrics.then((fn) => fn());
    };
  }, []);

  const data = metrics;
  const workloads = data?.ai_runtimes.workloads ?? [];

  return (
    <div className="tray-popup">
      <header className="tray-popup__header">
        <div>
          <strong>PulseMonitor</strong>
          <span className={`tray-popup__live ${connected ? "tray-popup__live--on" : ""}`}>
            {connected ? "Live" : "Connecting…"}
          </span>
        </div>
      </header>

      {data ? (
        <>
          <div className="tray-popup__bars">
            <MiniBar label="CPU" value={data.cpu.overall_usage} color="var(--accent)" />
            <MiniBar label="GPU" value={data.gpu.available ? data.gpu.usage_percent : 0} color="var(--cyan)" />
            <MiniBar label="RAM" value={data.memory.usage_percent} color="var(--warning)" />
          </div>

          <section className="tray-popup__models">
            <h3>AI Models</h3>
            {workloads.length === 0 ? (
              <p className="tray-popup__empty">No loaded models</p>
            ) : (
              workloads.slice(0, 4).map((w) => (
                <div key={`${w.runtime}-${w.model}`} className="tray-model-row">
                  <div>
                    <span className="tray-model-row__name">{w.model}</span>
                    <span className="tray-model-row__meta">
                      {w.accelerator} · {w.status}
                    </span>
                  </div>
                  <span className="tray-model-row__tps">
                    {w.tokens_per_sec != null && w.tokens_per_sec > 0
                      ? `${w.tokens_per_sec.toFixed(0)} tok/s${w.throughput_kind === "estimated" ? "~" : ""}`
                      : w.status === "running"
                        ? "Active"
                        : "Idle"}
                  </span>
                </div>
              ))
            )}
          </section>

          {data.ai_runtimes.bottleneck && (
            <div className="tray-popup__hint">{data.ai_runtimes.bottleneck.message}</div>
          )}
        </>
      ) : (
        <p className="tray-popup__empty">Waiting for metrics…</p>
      )}

      <footer className="tray-popup__actions">
        <button type="button" onClick={() => invoke("open_dashboard")}>
          Open Dashboard
        </button>
        <button type="button" className="tray-popup__quit" onClick={() => invoke("quit_app")}>
          Quit
        </button>
      </footer>
    </div>
  );
}
