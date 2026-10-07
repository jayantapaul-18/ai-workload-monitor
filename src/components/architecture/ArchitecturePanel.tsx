import { useState } from "react";
import type { MetricsSnapshot } from "../../types/metrics";

interface ArchitecturePanelProps {
  metrics: MetricsSnapshot;
}

type LayerId = "kernel" | "engine" | "ipc" | "ui";

interface LayerInfo {
  id: LayerId;
  number: string;
  name: string;
  subtitle: string;
  accent: string;
  badge: string;
  components: {
    title: string;
    description: string;
    source: string;
    metric: string;
  }[];
  flowSummary: string;
}

const LAYERS: LayerInfo[] = [
  {
    id: "kernel",
    number: "01",
    name: "Linux Kernel & Hardware Sensors",
    subtitle: "Direct virtual filesystem & driver telemetry",
    accent: "var(--cyan)",
    badge: "Low-Overhead Telemetry",
    flowSummary: "Direct virtual filesystem reads without daemon overhead",
    components: [
      {
        title: "ProcFS Telemetry Reader",
        description:
          "Reads /proc/stat, /proc/meminfo, /proc/uptime, and /proc/net/dev directly without spawning shell subprocesses.",
        source: "src-tauri/src/metrics/{cpu, memory, network}.rs",
        metric: "CPU cores, load avg, RAM/swap bytes, interface RX/TX",
      },
      {
        title: "NVIDIA SMI Driver Bridge",
        description:
          "Executes batched nvidia-smi query commands with CSV formatting to extract GPU usage %, VRAM bytes, temperature, power watts, and clock speeds.",
        source: "src-tauri/src/metrics/gpu.rs",
        metric: "PCIe NVIDIA GPUs (RTX, GTX, Tesla, A100/H100)",
      },
      {
        title: "Sysfs DRM & Accel Subsystems",
        description:
          "Reads /sys/class/drm for AMDGPU and Intel Xe/i915 cards, plus /sys/class/accel for AMD XDNA NPU driver states.",
        source: "src-tauri/src/metrics/{gpu, npu}.rs",
        metric: "AMDGPU, Intel Arc, AMD Ryzen AI NPU",
      },
      {
        title: "Process Table Inspector",
        description:
          "Parses /proc/[pid]/stat with normalized multi-core math, identifying active AI workloads and Ollama runners.",
        source: "src-tauri/src/metrics/process.rs",
        metric: "Live PIDs, CPU %, memory bytes, AI tag filter",
      },
    ],
  },
  {
    id: "engine",
    number: "02",
    name: "Rust Engine & Decoupled Async Worker",
    subtitle: "High-performance synchronization & correlation",
    accent: "var(--accent)",
    badge: "Decoupled Async Architecture",
    flowSummary: "Separates 1s system tick from 2s network runtime scraping",
    components: [
      {
        title: "MetricsEngine (1s Synchronous Tick)",
        description:
          "Fast system collector running on the main polling loop. Never makes blocking network calls. Reads system counters in < 1ms.",
        source: "src-tauri/src/metrics/mod.rs",
        metric: "1000ms base poll, 500ms burst on focus",
      },
      {
        title: "Decoupled AI Scraper Worker",
        description:
          "Dedicated background OS thread running independently. Polls Ollama, vLLM, llama.cpp, and ComfyUI every 2 seconds into a shared mutex cache.",
        source: "src-tauri/src/metrics/ai_runtimes.rs",
        metric: "Sub-microsecond lock read, zero UI freezing",
      },
      {
        title: "Live Correlation & Bottleneck Diagnosis",
        description:
          "Correlates host GPU usage with process CPU to calculate inference throughput and diagnose VRAM, thermal, or CPU bottlenecks.",
        source: "src-tauri/src/metrics/ai_runtimes.rs",
        metric: "Real-time bottleneck insights & TPS smoothing",
      },
      {
        title: "Alert Evaluator & Threshold Dispatcher",
        description:
          "Monitors CPU %, RAM %, GPU °C, and AI process changes against user-configurable thresholds with configurable cooldowns.",
        source: "src-tauri/src/lib.rs",
        metric: "OS desktop notifications & in-app alerts",
      },
    ],
  },
  {
    id: "ipc",
    number: "03",
    name: "Tauri IPC & System Bridge",
    subtitle: "Secure local communication & OS integration",
    accent: "var(--purple)",
    badge: "Tauri 2 IPC Core",
    flowSummary: "Bi-directional event streaming with zero cloud transmission",
    components: [
      {
        title: "Asynchronous Event Bridge",
        description:
          "Emits serialized MetricsSnapshot payloads over Tauri 2 event channels directly to Webview windows.",
        source: "src-tauri/src/lib.rs",
        metric: "metrics-update, alert-triggered, settings-changed",
      },
      {
        title: "Native System Tray Controller",
        description:
          "Manages dynamic tray tooltip with live CPU/GPU/RAM stats and controls the lightweight tray quick-view popup window.",
        source: "src-tauri/src/lib.rs",
        metric: "Global system tray, multi-window orchestration",
      },
      {
        title: "JSON Settings Store",
        description:
          "Atomic configuration persistence stored in ~/.config/pulse-monitor/settings.json with schema migrations and defaults.",
        source: "src-tauri/src/settings.rs",
        metric: "Custom URLs, thresholds, poll intervals, keywords",
      },
      {
        title: "Strict Content Security Policy (CSP)",
        description:
          "Enforces default-src 'self' to block external script injection, unauthorized network fetches, and external trackers.",
        source: "src-tauri/tauri.conf.json",
        metric: "100% offline, privacy-first sandboxed environment",
      },
    ],
  },
  {
    id: "ui",
    number: "04",
    name: "React 19 + TypeScript Presentation",
    subtitle: "Real-time visualization & responsive controls",
    accent: "var(--warning)",
    badge: "Hardware-Accelerated UI",
    flowSummary: "Pure client rendering optimized with microsecond hooks",
    components: [
      {
        title: "Overview Dashboard & Live Graphs",
        description:
          "SVG Sparklines, circular RingGauges, and CoreMatrix rendered efficiently using rolling ring-buffer histories.",
        source: "src/components/overview/{OverviewPanel, ...}.tsx",
        metric: "60fps animations, customizable history length",
      },
      {
        title: "Active AI Workloads Panel",
        description:
          "Visual workload cards displaying live models from Ollama, vLLM, llama.cpp, and ComfyUI with status, accelerator, VRAM, and tokens/sec.",
        source: "src/components/overview/ActiveAiWorkloads.tsx",
        metric: "Ollama, vLLM, llama.cpp, ComfyUI cards",
      },
      {
        title: "Sortable Process Inspector",
        description:
          "Full process table with instant multi-column sorting (PID, Name, CPU %, Memory) and real-time substring search filter.",
        source: "src/components/processes/ProcessPanel.tsx",
        metric: "Search, column sorting, AI workload badges",
      },
      {
        title: "Hardware & Settings Panels",
        description:
          "Comprehensive hardware topology inspection (PCIe devices, CPU features, NPU state) and runtime endpoint configuration.",
        source: "src/components/{hardware, settings}/*.tsx",
        metric: "Lspci bridge, kernel profile, theme switcher",
      },
    ],
  },
];

const RUNTIMES = [
  {
    name: "Ollama",
    badgeClass: "ai-workload-card__runtime--ollama",
    defaultPort: "11434",
    protocol: "HTTP REST API + CLI",
    endpoint: "GET /api/ps",
    extractedData: "Running models, VRAM bytes, parameter size, quantization, context length",
    status: "Active polling (2s)",
  },
  {
    name: "vLLM",
    badgeClass: "ai-workload-card__runtime--vllm",
    defaultPort: "8000",
    protocol: "Prometheus Metrics",
    endpoint: "GET /metrics",
    extractedData: "Generation tok/s, GPU cache usage %, active requests, model names",
    status: "Active polling (2s)",
  },
  {
    name: "llama.cpp",
    badgeClass: "ai-workload-card__runtime--llama_cpp",
    defaultPort: "8080",
    protocol: "HTTP Server Slots / Props",
    endpoint: "GET /slots, /props",
    extractedData: "Slot processing states, GGUF model path, context length, parameters",
    status: "Active polling (2s)",
  },
  {
    name: "ComfyUI",
    badgeClass: "ai-workload-card__runtime--comfyui",
    defaultPort: "8188",
    protocol: "HTTP Queue & Stats",
    endpoint: "GET /queue, /system_stats",
    extractedData: "Active workflow prompts, checkpoint name, device VRAM, queue counts",
    status: "Active polling (2s)",
  },
];

export function ArchitecturePanel({ metrics }: ArchitecturePanelProps) {
  const [activeLayer, setActiveLayer] = useState<LayerId>("engine");
  const selected = LAYERS.find((l) => l.id === activeLayer) ?? LAYERS[1];
  const runtimesCount = metrics.ai_runtimes.runtimes_online.length;
  const modelsCount = metrics.ai_runtimes.workloads.length;

  return (
    <div className="architecture-panel">
      <header className="panel-header">
        <div>
          <h2>System Architecture</h2>
          <p className="panel-subtitle">
            Running on {metrics.system.os || "Linux"} ({metrics.system.kernel || "Kernel"}) · Low-overhead kernel telemetry, decoupled async AI workers, and Tauri 2 IPC.
          </p>
        </div>
        <div className="architecture-stats">
          <div className="architecture-stat">
            <span className="architecture-stat__label">Engine Latency</span>
            <span className="architecture-stat__value">&lt; 1 ms</span>
          </div>
          <div className="architecture-stat">
            <span className="architecture-stat__label">Async AI Worker</span>
            <span className="architecture-stat__value">2000 ms</span>
          </div>
          <div className="architecture-stat">
            <span className="architecture-stat__label">Connected Runtimes</span>
            <span className="architecture-stat__value">{runtimesCount} Online</span>
          </div>
          <div className="architecture-stat">
            <span className="architecture-stat__label">Active Models</span>
            <span className="architecture-stat__value">{modelsCount} Loaded</span>
          </div>
        </div>
      </header>

      {/* Visual Pipeline Flow */}
      <section className="architecture-flow">
        <h3 className="section-title">Multi-Tier Data Pipeline</h3>
        <div className="architecture-pipeline">
          {LAYERS.map((layer, idx) => {
            const isSelected = layer.id === activeLayer;
            return (
              <div
                key={layer.id}
                className={`pipeline-tier ${isSelected ? "pipeline-tier--active" : ""}`}
                onClick={() => setActiveLayer(layer.id)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") setActiveLayer(layer.id);
                }}
              >
                <div className="pipeline-tier__header">
                  <span className="pipeline-tier__number">{layer.number}</span>
                  <span className="pipeline-tier__badge" style={{ borderColor: layer.accent, color: layer.accent }}>
                    {layer.badge}
                  </span>
                </div>
                <h4 className="pipeline-tier__title">{layer.name}</h4>
                <p className="pipeline-tier__desc">{layer.subtitle}</p>
                <div className="pipeline-tier__footer">
                  <span className="pipeline-tier__click">Click to inspect layer →</span>
                </div>
                {idx < LAYERS.length - 1 && <div className="pipeline-tier__arrow">→</div>}
              </div>
            );
          })}
        </div>
      </section>

      {/* Deep-Dive Inspection Card */}
      <section className="architecture-detail">
        <div className="architecture-detail__header" style={{ borderLeftColor: selected.accent }}>
          <div>
            <div className="architecture-detail__pre">
              LAYER {selected.number} · {selected.badge}
            </div>
            <h3>{selected.name}</h3>
            <p className="architecture-detail__summary">{selected.flowSummary}</p>
          </div>
          <div className="architecture-layer-tabs">
            {LAYERS.map((l) => (
              <button
                key={l.id}
                className={`layer-btn ${l.id === activeLayer ? "layer-btn--active" : ""}`}
                onClick={() => setActiveLayer(l.id)}
              >
                {l.number} {l.name.split(" ")[0]}
              </button>
            ))}
          </div>
        </div>

        <div className="architecture-components-grid">
          {selected.components.map((comp) => (
            <div key={comp.title} className="architecture-component-card">
              <div className="architecture-component-card__top">
                <h4>{comp.title}</h4>
                <code>{comp.source}</code>
              </div>
              <p className="architecture-component-card__desc">{comp.description}</p>
              <div className="architecture-component-card__meta">
                <span className="meta-label">Payload / Scope:</span>
                <span className="meta-val">{comp.metric}</span>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* AI Runtime Matrix */}
      <section className="architecture-runtimes">
        <h3 className="section-title">Supported AI Runtimes & Polling Strategy</h3>
        <p className="section-desc">
          AI runtime scraping is completely decoupled from the fast 1s system metrics collection loop. Remote timeouts or slow endpoints never stall the UI.
        </p>
        <div className="runtime-table-wrap">
          <table className="runtime-table">
            <thead>
              <tr>
                <th>Runtime</th>
                <th>Port</th>
                <th>Protocol</th>
                <th>Endpoint</th>
                <th>Extracted Metadata</th>
                <th>Strategy</th>
              </tr>
            </thead>
            <tbody>
              {RUNTIMES.map((rt) => (
                <tr key={rt.name}>
                  <td>
                    <span className={`ai-workload-card__runtime ${rt.badgeClass}`}>
                      {rt.name}
                    </span>
                  </td>
                  <td><code>:{rt.defaultPort}</code></td>
                  <td>{rt.protocol}</td>
                  <td><code>{rt.endpoint}</code></td>
                  <td>{rt.extractedData}</td>
                  <td>
                    <span className="runtime-strategy-badge">{rt.status}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Technical Guarantees */}
      <section className="architecture-guarantees">
        <div className="guarantee-card">
          <div className="guarantee-icon">⚡</div>
          <h4>Decoupled Async Polling</h4>
          <p>
            System metrics (CPU, RAM, GPU, network) are collected every 1 second in microsecond time. AI runtime scraping runs on a decoupled background thread every 2 seconds, preventing remote timeouts from blocking system telemetry.
          </p>
        </div>
        <div className="guarantee-card">
          <div className="guarantee-icon">🛡️</div>
          <h4>Zero External Telemetry</h4>
          <p>
            All monitoring is 100% offline and strictly local to your machine (127.0.0.1). No analytics, no remote pings, and no cloud dependencies.
          </p>
        </div>
        <div className="guarantee-card">
          <div className="guarantee-icon">🎯</div>
          <h4>Real-Time Bottleneck AI</h4>
          <p>
            Dynamically correlates GPU compute, VRAM allocation, and host CPU usage to diagnose pipeline bottlenecks like thermal throttling and low-throughput stalls.
          </p>
        </div>
      </section>
    </div>
  );
}
