export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(decimals)} ${sizes[i]}`;
}

export function formatRate(bytesPerSec: number): string {
  return `${formatBytes(bytesPerSec)}/s`;
}

export const formatBytesPerSec = formatRate;

export function formatPercent(value: number, decimals = 0): string {
  return `${value.toFixed(decimals)}%`;
}

export function formatUptime(secs: number): string {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function usageColor(percent: number): string {
  if (percent >= 85) return "var(--danger)";
  if (percent >= 60) return "var(--warning)";
  return "var(--accent)";
}

export function usageClass(percent: number): string {
  if (percent >= 85) return "hot";
  if (percent >= 60) return "warm";
  return "cool";
}

export function usageBg(usage: number): string {
  if (usage >= 85) return "var(--danger-dim)";
  if (usage >= 60) return "var(--warning-dim)";
  return "var(--accent-dim)";
}

export function truncate(str: string, max: number): string {
  if (str.length <= max) return str;
  return `${str.slice(0, max - 1)}…`;
}
