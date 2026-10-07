# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-10-06

### Added
- **ComfyUI Integration**: Real-time queue depth tracking, active generation prompt parsing, checkpoint/model name extraction (e.g. `safetensors`), and device VRAM monitoring via `/queue` and `/system_stats`.
- **llama.cpp Support**: Live `llama-server` telemetry including slot processing states, active running requests, loaded GGUF model path, parameter size, quantization, and context length via `/slots` and `/props`.
- **Decoupled Asynchronous AI Scraper**: Isolated all external AI runtime HTTP polling onto an independent background worker thread running every 2 seconds. The 1-second system collection loop now reads cached metrics in under a microsecond, eliminating UI freezing and chart stuttering from remote timeouts or network latency.
- **Interactive System Architecture Explorer**: New dedicated Architecture tab in the sidebar featuring an interactive 4-tier data pipeline diagram (Kernel &rarr; Engine &rarr; IPC &rarr; UI), component inspect cards, an AI runtime integration matrix, and technical guarantees.
- **NVIDIA GPU Telemetry**: Automatic detection and real-time hardware telemetry (GPU usage %, VRAM bytes, temperature, power watts, and clock speeds) via batched `nvidia-smi` CSV queries.
- **Process Table Search & Multi-Column Sorting**: Live substring search filter and sortable column headers for PID, Name, CPU %, and Memory bytes.
- **Runtime Connection Status Bar**: Live indicator pills on the Overview dashboard displaying connection states, port mappings, and active model counts for Ollama, vLLM, llama.cpp, and ComfyUI.
- **Compact Tray Quick-View Popup**: Interactive system tray popup window with mini progress bars for CPU, GPU, RAM, and loaded AI model rows.
- **Strict Content Security Policy (CSP)**: Hardened desktop environment enforcing `default-src 'self'` for 100% offline, privacy-first execution with zero external script execution.

### Fixed & Improved
- Fixed Ollama runner CPU 0% calculation bug by reusing a persistent `ProcessCollector` instance across polling ticks.
- Batched and cached network interface IPv4 address lookups, reducing external process forks by 99%.
- Dynamically wired configurable rolling history length (`historyLength`) from Settings into all dashboard charts.
- Updated branding across the application, sidebar, tray, and settings.

---

## [0.1.0] - 2026-10-05

### Added
- First public release.
- Linux desktop monitor for CPU, GPU, NPU, memory, processes, and network.
- Ollama and vLLM workload views with live bottleneck diagnostics.
- System tray with live hover tooltip, desktop alerts, dark/light themes, customizable accent colors, and settings.
- User install and login autostart scripts, plus `.deb`, AppImage, and RPM bundle support.
