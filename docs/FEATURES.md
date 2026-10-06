# Features

What AI Workload Monitor does today. Future direction lives in [ROADMAP.md](./ROADMAP.md).

## Monitoring

- CPU utilization per core, load average, and frequency
- AMD GPU utilization, temperature, power, and VRAM
- AMD XDNA NPU presence, power state, firmware, and driver
- RAM and swap, with overview sparklines
- Process lists for CPU, memory, background, and AI-tagged workloads
- Network interfaces, Wi-Fi details, and live throughput
- Live header for CPU, memory, GPU, and AI activity

## AI workloads

- Ollama connector for loaded models, VRAM, accelerator, and quantization
- Estimated tokens per second while Ollama is generating
- vLLM connector for measured tokens per second, running requests, and GPU cache
- Active AI Workloads cards on the overview
- Bottleneck hints for VRAM pressure, heat, RAM pressure, CPU-bound runs, and low throughput
- Configurable Ollama and vLLM URLs

## App

- Overview, CPU, GPU, NPU, processes, network, and hardware pages
- Hardware and runtime specs
- Settings for general behavior, polling, alerts, AI, appearance, and advanced options
- Dark and light themes, compact sidebar, and accent colors
- In-app alert toasts and desktop notifications
- JSON export of the latest metrics snapshot
- System tray icon, live tooltip, close-to-tray, start minimized, and compact tray popup
- Single-instance behavior: launching again focuses the existing window

## Install

- User install script and login autostart script
- `.deb`, AppImage, and RPM bundles from the Tauri release build
- MIT license
