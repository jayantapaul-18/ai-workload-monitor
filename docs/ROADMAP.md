# Roadmap

High-level direction for AI Workload Monitor. Shipped behavior is listed in [FEATURES.md](./FEATURES.md).

## Direction

A local monitor that understands the models and accelerators on one machine, and points at what is holding a run back.

## Completed Milestones

- Local AI runtime monitoring for **Ollama**, **vLLM**, **llama.cpp (`llama-server`)**, and **ComfyUI**
- Decoupled asynchronous polling worker ensuring sub-microsecond engine reads and zero UI stuttering
- NVIDIA GPU detection and telemetry via `nvidia-smi` (utilization, VRAM, temp, power, clocks)
- Interactive System Architecture & Data Pipeline inspection page
- Live process search filter and multi-column sorting
- Strict Content Security Policy (CSP) for 100% offline security

## Next

- History of inference and training sessions (peak VRAM, duration, average throughput)
- Measured throughput aggregation across multiple concurrent server slots
- Custom AI process tagging rules in Settings

## Later

- Model fit advisory: evaluate whether a GGUF/safetensors model will fit current GPU VRAM before loading
- Container-aware process grouping (Docker, Podman, Kubernetes cgroups)
- OpenTelemetry / Prometheus exporter endpoint for scraping this monitor's metrics
- Deeper per-process accelerator attribution via NVML / ROCm SMI
