# AI Workload Monitor — Design Spec

**Date:** 2026-06-21  
**Stack:** Tauri 2 + Rust collectors + React UI  
**Target:** Ubuntu local monitor for AI workloads (AMD Ryzen 8845HS + Radeon 780M + XDNA NPU)

## Architecture

- **Rust MetricsEngine** polls `/proc`, `/sys`, and `sysinfo` on a 1 Hz tokio loop
- **Tauri events** push `MetricsSnapshot` JSON to the React dashboard
- **System tray** shows live CPU/GPU/RAM/network tooltip; close hides window

## Panels

Overview, CPU (per-core grid), GPU, NPU, Processes (top CPU/mem/background/AI), Network (WiFi + interfaces), System

## Performance

Single aggregated snapshot per tick, delta-based CPU/network, cached static system info, no subprocess polling loops.
