export interface AppSettings {
  startMinimized: boolean;
  closeToTray: boolean;
  launchAtLogin: boolean;
  showTrayTooltip: boolean;
  trayPopupEnabled: boolean;
  pollIntervalMs: number;
  burstPollWhenFocused: boolean;
  processLimit: number;
  historyLength: number;
  alertsEnabled: boolean;
  cpuAlertThreshold: number;
  memoryAlertThreshold: number;
  gpuTempAlertThreshold: number;
  alertCooldownSecs: number;
  notifyAiProcessChanges: boolean;
  aiKeywords: string[];
  highlightAiInProcesses: boolean;
  autoFocusAiTab: boolean;
  aiIntelligenceEnabled: boolean;
  ollamaUrl: string;
  vllmMetricsUrl: string;
  colorTheme: "dark" | "light";
  accentTheme: "green" | "blue" | "purple";
  compactSidebar: boolean;
  compactDensity: boolean;
  showSparklines: boolean;
  animateGauges: boolean;
  exportIncludeProcesses: boolean;
  lowPowerMode: boolean;
}

export type SettingsSection =
  | "general"
  | "monitoring"
  | "alerts"
  | "ai"
  | "appearance"
  | "advanced"
  | "about";

export const DEFAULT_SETTINGS: AppSettings = {
  startMinimized: false,
  closeToTray: true,
  launchAtLogin: false,
  showTrayTooltip: true,
  trayPopupEnabled: true,
  pollIntervalMs: 1000,
  burstPollWhenFocused: true,
  processLimit: 12,
  historyLength: 60,
  alertsEnabled: true,
  cpuAlertThreshold: 90,
  memoryAlertThreshold: 90,
  gpuTempAlertThreshold: 85,
  alertCooldownSecs: 60,
  notifyAiProcessChanges: false,
  aiKeywords: [
    "python", "ollama", "llama", "vllm", "torch", "docker", "jupyter",
    "transformers", "comfyui", "fastapi", "uvicorn", "node", "xinference",
  ],
  highlightAiInProcesses: true,
  autoFocusAiTab: false,
  aiIntelligenceEnabled: true,
  ollamaUrl: "http://127.0.0.1:11434",
  vllmMetricsUrl: "http://127.0.0.1:8000/metrics",
  colorTheme: "dark",
  accentTheme: "green",
  compactSidebar: false,
  compactDensity: false,
  showSparklines: true,
  animateGauges: true,
  exportIncludeProcesses: true,
  lowPowerMode: false,
};

export const POLL_OPTIONS = [
  { value: 250, label: "250 ms (Ultra)" },
  { value: 500, label: "500 ms (Fast)" },
  { value: 1000, label: "1 s (Default)" },
  { value: 2000, label: "2 s (Eco)" },
  { value: 5000, label: "5 s (Low power)" },
];

export const THEME_OPTIONS = [
  { value: "dark" as const, label: "Dark", icon: "🌙" },
  { value: "light" as const, label: "Light", icon: "☀" },
];

export const ACCENT_OPTIONS = [
  { value: "green" as const, label: "Emerald", color: "#3dd68c" },
  { value: "blue" as const, label: "Cyan", color: "#45caff" },
  { value: "purple" as const, label: "Violet", color: "#a78bfa" },
];
