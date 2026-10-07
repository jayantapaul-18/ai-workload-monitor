# AI Workload Monitor

Local system monitor for Linux, built for developers running models and AI workflows on their own hardware. It monitors CPU, GPU, NPU, memory, processes, and network, and recognizes **Ollama**, **vLLM**, **llama.cpp (`llama-server`)**, and **ComfyUI** workloads in the same view.

Tauri 2 collects kernel and driver metrics in Rust with a decoupled background scraper. React 19 renders the hardware-accelerated dashboard.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platform: Linux](https://img.shields.io/badge/Platform-Linux-FCC624?logo=linux&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-2021-DEA584?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB?logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6?logo=typescript&logoColor=white)
![Vite](https://img.shields.io/badge/Vite-7-646CFF?logo=vite&logoColor=white)
![NVIDIA](https://img.shields.io/badge/NVIDIA-CUDA-76B900?logo=nvidia&logoColor=white)
![AMD](https://img.shields.io/badge/AMD-ROCm-ED1C24?logo=amd&logoColor=white)
![Ollama](https://img.shields.io/badge/Runtime-Ollama-black?logo=ollama&logoColor=white)
![vLLM](https://img.shields.io/badge/Runtime-vLLM-blueviolet)
![llama.cpp](https://img.shields.io/badge/Runtime-llama.cpp-orange)
![ComfyUI](https://img.shields.io/badge/Runtime-ComfyUI-10b981)

By [Jayanta Paul](https://github.com/jayantapaul-18).

## Screenshots

![Performance Overview](docs/performance-overview.png)

![CPU cores](docs/cpu-cores.png)

![Processes and active AI workloads](docs/processes.png)

## Architecture Overview

AI Workload Monitor uses a 4-tier architecture designed for minimal host footprint and zero UI freezing:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        React 19 + TypeScript UI                        │
│  Overview · CPU · GPU · NPU · Processes · Network · Hardware · Arch    │
└───────────────────────────────────▲────────────────────────────────────┘
                                    │ Tauri 2 IPC (metrics-update)
┌───────────────────────────────────┴────────────────────────────────────┐
│                       Rust Engine & System Bridge                      │
│                                                                        │
│   ┌───────────────────────────┐      ┌─────────────────────────────┐   │
│   │ MetricsEngine (1s Loop)   │◄─────┤ Decoupled Async AI Worker   │   │
│   │  • Low-latency sysfs      │ Read │  • Background thread (2s)   │   │
│   │  • Delta samplers         │Cache │  • Zero UI thread blocking  │   │
│   │  • Bottleneck correlation │      │  • Timeout resilient (800ms)│   │
│   └─────────────▲─────────────┘      └──────────────▲──────────────┘   │
└─────────────────┼───────────────────────────────────┼──────────────────┘
                  │                                   │
┌─────────────────┴─────────────┐      ┌──────────────┴──────────────┐
│    Kernel & Driver Telemetry  │      │     Local AI Runtimes       │
│  • /proc/{stat, meminfo, dev} │      │  • Ollama (/api/ps, CLI)    │
│  • nvidia-smi CSV driver      │      │  • vLLM (/metrics)          │
│  • sysfs DRM (AMDGPU, Intel)  │      │  • llama.cpp (/slots,/props)│
│  • sysfs Accel (AMD XDNA NPU) │      │  • ComfyUI (/queue, /stats) │
└───────────────────────────────┘      └─────────────────────────────┘
```

The app features an interactive **System Architecture** page inside the UI (accessible via the sidebar) showing live pipeline latencies, component mappings, and runtime integration protocols.

## Key Features

- **Decoupled Asynchronous AI Polling**: AI runtime HTTP requests are isolated to a background thread every 2s. Slow endpoints, network latency, or server timeouts never block the 1s system telemetry loop.
- **Broad AI Runtime Support**:
  - **Ollama**: Running models, VRAM bytes, accelerator, quant, and estimated generation tok/s.
  - **vLLM**: Measured generation throughput (tok/s), GPU cache usage %, and active requests via Prometheus metrics.
  - **llama.cpp (`llama-server`)**: Slot processing states, GGUF model path, context length, parameters, and quants via `/slots` and `/props`.
  - **ComfyUI**: Real-time generation status, active workflow checkpoint/model, VRAM usage, and queue depth via `/queue` and `/system_stats`.
- **GPU Telemetry**:
  - **NVIDIA**: Automatic detection and live metrics via `nvidia-smi` (usage %, VRAM used/total, temperature °C, power watts, core clock MHz).
  - **AMD & Intel**: Direct sysfs DRM telemetry (`/sys/class/drm/card*`).
- **NPU Awareness**: AMD XDNA Ryzen AI NPU driver state, firmware version, and power status.
- **Process Inspector**: Real-time process listing with substring search filter and sortable columns (PID, Name, CPU %, Memory, AI tag).
- **Bottleneck Diagnosis**: Correlates GPU load with host CPU and model memory to flag VRAM exhaustion, thermal throttling, and CPU-bound offloading.
- **Hardware Profile & System Architecture Pages**: Comprehensive PCIe topology, CPU instruction set features, and interactive architectural data flow diagrams.
- **Privacy & Security**: 100% offline local monitoring (127.0.0.1 loopback only), zero external telemetry, and strict Content Security Policy (CSP).

The complete feature catalog is in [docs/FEATURES.md](docs/FEATURES.md). Future direction is tracked in [docs/ROADMAP.md](docs/ROADMAP.md).

## Requirements

- Ubuntu 22.04 or newer (WebKitGTK 4.1)
- Rust stable, via [rustup](https://rustup.rs)
- Node.js 20.19 or newer
- Optional local AI runtimes:
  - [Ollama](https://ollama.com) (`http://127.0.0.1:11434`)
  - [vLLM](https://github.com/vllm-project/vllm) (`http://127.0.0.1:8000/metrics`)
  - [llama.cpp](https://github.com/ggerganov/llama.cpp) `llama-server` (`http://127.0.0.1:8080`)
  - [ComfyUI](https://github.com/comfyanonymous/ComfyUI) (`http://127.0.0.1:8188`)

## Quick start

```bash
chmod +x scripts/*.sh
./scripts/install-deps.sh   # Ubuntu/Debian packages, needs sudo
npm install
npm run tauri dev
```

## Install

Build a release and install it for the current user (app menu and `~/.local/bin`):

```bash
./scripts/install.sh
```

Start on login:

```bash
./scripts/install-autostart.sh
```

Remove the user install:

```bash
./scripts/uninstall.sh
```

`npm run tauri build` also writes installers under `src-tauri/target/release/bundle/` (`.deb`, AppImage, RPM).

## Tray

Closing the window hides the app in the tray. A left click opens the compact popup with CPU, GPU, RAM mini-bars and loaded AI models. The tray menu can show the dashboard, hide the window, or quit.

## Development

| Command | What it does |
| --- | --- |
| `npm run tauri dev` | Desktop app with hot reload |
| `npm run typecheck` | TypeScript check |
| `npm run tauri build` | Production bundles |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit tests |
| `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | Rust linting |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Rust formatting check |
| `npm run commit` | Commitizen prompt (conventional commits) |

Commits are validated by Husky pre-commit hooks (`npm run typecheck` and `cargo fmt --check`) and commit-msg hooks (Commitlint). See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
