import { useEffect, useState } from "react";
import { useMetrics } from "./hooks/useMetrics";
import { useSettings } from "./hooks/useSettings";
import { EMPTY_METRICS, TabId } from "./types/metrics";
import { Sidebar } from "./components/layout/Sidebar";
import { Header } from "./components/layout/Header";
import { OverviewPanel } from "./components/overview/OverviewPanel";
import { CpuPanel } from "./components/cpu/CpuPanel";
import { GpuPanel } from "./components/gpu/GpuPanel";
import { NpuPanel } from "./components/npu/NpuPanel";
import { ProcessPanel } from "./components/processes/ProcessPanel";
import { NetworkPanel } from "./components/network/NetworkPanel";
import { useHardware } from "./hooks/useHardware";
import { HardwarePanel } from "./components/hardware/HardwarePanel";
import { ArchitecturePanel } from "./components/architecture/ArchitecturePanel";
import { SettingsPanel } from "./components/settings/SettingsPanel";
import { AlertToasts } from "./components/shared/AlertToasts";
import "./App.css";

function App() {
  const [tab, setTab] = useState<TabId>("overview");
  const { settings, settingsPath, saving, update, reset } = useSettings();
  const { metrics, connected, history, gpuHistory, memHistory, rxHistory, txHistory } =
    useMetrics(settings.historyLength);
  const { profile, loading: hardwareLoading, error: hardwareError } = useHardware();
  const data = metrics ?? EMPTY_METRICS;

  useEffect(() => {
    const handler = () => setTab("settings");
    window.addEventListener("navigate-settings", handler);
    return () => window.removeEventListener("navigate-settings", handler);
  }, []);

  return (
    <div className="app">
      <Sidebar
        active={tab}
        onChange={setTab}
        aiCount={data.processes.ai_workloads.length}
        compact={settings.compactSidebar}
      />

      <div className="main">
        <Header metrics={metrics} connected={connected} />
        <AlertToasts />

        <div className="content">
          {tab === "overview" && (
            <OverviewPanel
              metrics={data}
              history={history}
              gpuHistory={gpuHistory}
              memHistory={memHistory}
              rxHistory={rxHistory}
              txHistory={txHistory}
            />
          )}
          {tab === "cpu" && <CpuPanel cpu={data.cpu} history={history} />}
          {tab === "gpu" && <GpuPanel gpu={data.gpu} history={gpuHistory} />}
          {tab === "npu" && <NpuPanel npu={data.npu} />}
          {tab === "processes" && (
            <ProcessPanel processes={data.processes} aiRuntimes={data.ai_runtimes} />
          )}
          {tab === "network" && (
            <NetworkPanel
              network={data.network}
              rxHistory={rxHistory}
              txHistory={txHistory}
            />
          )}
          {tab === "hardware" && (
            <HardwarePanel
              profile={profile}
              loading={hardwareLoading}
              error={hardwareError}
              metrics={data}
              settings={settings}
              settingsPath={settingsPath}
            />
          )}
          {tab === "architecture" && <ArchitecturePanel metrics={data} />}
          {tab === "settings" && (
            <SettingsPanel
              settings={settings}
              settingsPath={settingsPath}
              saving={saving}
              onUpdate={update}
              onReset={reset}
            />
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
