import type { AppSettings } from "../../types/settings";
import type { HardwareProfile } from "../../types/hardware";
import type { MetricsSnapshot } from "../../types/metrics";
import { formatBytes, formatUptime } from "../../utils/format";

interface HardwarePanelProps {
  profile: HardwareProfile | null;
  loading: boolean;
  error: string | null;
  metrics: MetricsSnapshot;
  settings: AppSettings;
  settingsPath: string;
}

interface SpecCardProps {
  icon: string;
  title: string;
  subtitle: string;
  status?: { label: string; ok: boolean };
  rows: [string, string][];
}

function SpecCard({ icon, title, subtitle, status, rows }: SpecCardProps) {
  return (
    <section className="hw-card">
      <div className="hw-card__head">
        <div className="hw-card__icon">{icon}</div>
        <div className="hw-card__titles">
          <h3>{title}</h3>
          <p>{subtitle}</p>
        </div>
        {status && (
          <span className={`hw-badge ${status.ok ? "hw-badge--ok" : "hw-badge--off"}`}>
            {status.label}
          </span>
        )}
      </div>
      <dl className="hw-card__rows">
        {rows.map(([label, value]) => (
          <div key={label} className="hw-row">
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

export function HardwarePanel({
  profile,
  loading,
  error,
  metrics,
  settings,
  settingsPath,
}: HardwarePanelProps) {
  if (loading) {
    return (
      <div className="panel hardware-panel">
        <div className="panel__header">
          <h2>Hardware & Runtime</h2>
          <p>Loading system specifications…</p>
        </div>
      </div>
    );
  }

  if (error || !profile) {
    return (
      <div className="panel hardware-panel">
        <div className="panel__header">
          <h2>Hardware & Runtime</h2>
          <p className="hw-error">Could not load hardware profile{error ? `: ${error}` : ""}</p>
        </div>
      </div>
    );
  }

  const { platform, cpu, gpu, npu, memory, storage, app } = profile;
  const pollLabel =
    settings.pollIntervalMs >= 1000
      ? `${settings.pollIntervalMs / 1000} s`
      : `${settings.pollIntervalMs} ms`;

  return (
    <div className="panel hardware-panel">
      <div className="panel__header">
        <h2>Hardware & Runtime</h2>
        <p>Static specifications and live monitoring configuration for this workstation</p>
      </div>

      <div className="hw-grid">
        <SpecCard
          icon="⌂"
          title="Platform"
          subtitle={platform.os}
          rows={[
            ["Hostname", platform.hostname],
            ["Kernel", platform.kernel],
            ["Architecture", platform.architecture],
            ["Desktop", platform.desktop],
            ["Uptime", formatUptime(metrics.system.uptime_secs)],
          ]}
        />

        <SpecCard
          icon="▦"
          title="Processor"
          subtitle={cpu.vendor || "CPU"}
          status={{ label: `${cpu.cores}c / ${cpu.threads}t`, ok: true }}
          rows={[
            ["Model", cpu.model],
            ["Max Frequency", cpu.max_frequency_mhz ? `${cpu.max_frequency_mhz} MHz` : "—"],
            ["L3 Cache", cpu.cache_l3_kb ? `${cpu.cache_l3_kb} KB` : "—"],
            ["Virtualization", cpu.virtualization],
            ["ISA Extensions", cpu.flags.length ? cpu.flags.join(", ") : "—"],
            ["Load Average", metrics.cpu.load_avg.map((v) => v.toFixed(2)).join(" · ")],
          ]}
        />

        <SpecCard
          icon="◈"
          title="Graphics"
          subtitle={gpu.detected ? gpu.driver : "No discrete/iGPU driver"}
          status={{ label: gpu.detected ? "Detected" : "Not found", ok: gpu.detected }}
          rows={[
            ["Device", gpu.name],
            ["Driver", gpu.driver || "—"],
            ["PCI Address", gpu.pci_address || "—"],
            ["VRAM", gpu.vram_bytes ? formatBytes(gpu.vram_bytes) : "—"],
            ["Render Nodes", gpu.render_nodes.length ? gpu.render_nodes.join(", ") : "—"],
            ["Live Utilization", metrics.gpu.available ? `${metrics.gpu.usage_percent.toFixed(0)}%` : "—"],
          ]}
        />

        <SpecCard
          icon="⬡"
          title="NPU Accelerator"
          subtitle={npu.detected ? npu.driver : "AMD XDNA / amdxdna"}
          status={{
            label: npu.detected ? npu.power_state : "Offline",
            ok: npu.detected,
          }}
          rows={[
            ["Device", npu.name],
            ["Firmware", npu.firmware || "—"],
            ["PCI Address", npu.pci_address || "—"],
            ["Utilization API", npu.utilization_available ? "Available" : "Not exposed by kernel"],
            [
              "Live Usage",
              metrics.npu.usage_percent != null
                ? `${metrics.npu.usage_percent.toFixed(0)}%`
                : npu.detected
                  ? "Online (no % metric)"
                  : "—",
            ],
          ]}
        />

        <SpecCard
          icon="▤"
          title="Memory"
          subtitle={formatBytes(memory.total_bytes)}
          rows={[
            ["Total RAM", formatBytes(memory.total_bytes)],
            ["Swap", formatBytes(memory.swap_bytes)],
            ["Layout", memory.channels],
            ["Available Now", formatBytes(metrics.memory.available_bytes)],
            ["Cached", formatBytes(metrics.memory.cached_bytes)],
            ["Live Usage", `${metrics.memory.usage_percent.toFixed(1)}%`],
          ]}
        />

        {storage.length > 0 && (
          <SpecCard
            icon="▣"
            title="Storage"
            subtitle={`${storage.length} device${storage.length > 1 ? "s" : ""}`}
            rows={storage.flatMap((disk) => [
              [`${disk.name.toUpperCase()} · ${disk.kind}`, formatBytes(disk.size_bytes)],
              [`${disk.name} model`, disk.model || "—"],
            ])}
          />
        )}

        <SpecCard
          icon="⚙"
          title="Runtime Configuration"
          subtitle={`${app.name} v${app.version}`}
          status={{ label: "Local monitor", ok: true }}
          rows={[
            ["App Version", app.version],
            ["Bundle ID", app.identifier],
            ["Poll Interval", pollLabel],
            ["Burst When Focused", settings.burstPollWhenFocused ? "On" : "Off"],
            ["Low Power Mode", settings.lowPowerMode ? "On" : "Off"],
            ["Process Sample Limit", String(settings.processLimit)],
            ["AI Keywords", `${settings.aiKeywords.length} patterns`],
            ["Alerts", settings.alertsEnabled ? "Enabled" : "Disabled"],
            ["Theme", `${settings.colorTheme} · ${settings.accentTheme}`],
            ["Settings File", settingsPath || "~/.config/ai-workload-monitor/settings.json"],
          ]}
        />
      </div>
    </div>
  );
}
