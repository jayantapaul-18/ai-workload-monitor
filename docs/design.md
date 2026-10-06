# AI Workload Monitor — Design Spec

**Date:** 2026-06-21  
**Stack:** Tauri 2 + Rust + React  
**Target:** Ubuntu local AI workload monitoring

## Goal

High-performance desktop monitor with per-core CPU, GPU (AMD amdgpu), NPU (AMD XDNA), memory, processes (AI-tagged), network, and system tray mini status.

## Architecture

- **Rust `MetricsEngine`** — single background thread, 1 Hz polling
- **Delta metrics** — CPU from `/proc/stat`, network from `/proc/net/dev`
- **Direct sysfs** — GPU (`gpu_busy_percent`, hwmon), NPU (`/sys/class/accel`)
- **sysinfo** — process list (2 s effective refresh)
- **Tauri events** — push `MetricsSnapshot` to React UI
- **System tray** — tooltip with CPU/GPU/RAM/AI count; click toggles window; close hides to tray

## UI Tabs

Overview · CPU · GPU · NPU · Processes · Network · System

## Performance

- No subprocess polling loops (WiFi SSID via `iw`/`nmcli` every 10 s only)
- UI throttled to 1 Hz via event rate
- Static system info cached at startup

## NPU Limitation

AMD XDNA exposes device health via sysfs; runtime utilization pending kernel/XRT support.
