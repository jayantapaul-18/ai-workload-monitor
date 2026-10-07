# Features

What AI Workload Monitor does today. Future direction lives in [ROADMAP.md](./ROADMAP.md).

## Monitoring

- CPU utilization per core, load average, and frequency
- NVIDIA GPU telemetry via `nvidia-smi` (utilization %, VRAM used/total, temperature, power, clock speed)
- AMD & Intel GPU utilization, temperature, power, and VRAM via sysfs DRM
- AMD XDNA NPU presence, power state, firmware, and driver
- RAM and swap, with overview sparklines
- Process lists for CPU, memory, background, and AI-tagged workloads
- Real-time process search filtering and multi-column sorting (PID, Name, CPU, RAM)
- Network interfaces, Wi-Fi details, and live throughput
- Live header for CPU, memory, GPU, and AI activity

## AI Workloads & Architecture

- **Decoupled Asynchronous Polling**: AI runtime HTTP scraping runs in a dedicated background worker every 2s, completely decoupled from the 1s system metrics collection loop (zero UI freezing on remote timeouts)
- **Ollama connector**: loaded models, VRAM, accelerator, quantization, and estimated tokens per second
- **vLLM connector**: measured tokens per second, running requests, and GPU cache usage % via Prometheus
- **llama.cpp (`llama-server`) connector**: slot processing status, loaded GGUF model path, parameters, quantization, context length, and active request count
- **ComfyUI connector**: active generation status, running prompt checkpoint/model, device VRAM, and queue depth
- **Active AI Workloads cards**: live overview cards with accelerator, memory, context, and throughput metrics
- **Bottleneck hints**: real-time diagnosis for VRAM pressure, thermal throttling, RAM pressure, CPU-bound runs, and low throughput
- Configurable endpoints for Ollama, vLLM, llama.cpp, and ComfyUI

## App & Interface

- Overview, CPU, GPU, NPU, processes, network, hardware, and architecture pages
- **System Architecture Page**: interactive 4-tier pipeline explorer (Kernel, Engine, IPC, UI), runtime integration matrix, and technical guarantees
- Hardware and runtime specs page
- Settings for general behavior, polling intervals, alert thresholds, AI endpoints, appearance, and advanced options
- Dark and light themes, compact sidebar, and accent colors
- In-app alert toasts and desktop notifications
- JSON export of the latest metrics snapshot
- System tray icon, live tooltip, close-to-tray, start minimized, and compact tray popup
- Strict Content Security Policy (CSP) for 100% offline, privacy-first sandboxing
- Single-instance behavior: launching again focuses the existing window

## Install

- User install script and login autostart script
- `.deb`, AppImage, and RPM bundles from the Tauri release build
- MIT license
