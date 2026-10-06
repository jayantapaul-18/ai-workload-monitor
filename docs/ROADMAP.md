# Roadmap

High-level direction for AI Workload Monitor. Shipped behavior is listed in [FEATURES.md](./FEATURES.md).

## Direction

A local monitor that understands the models and accelerators on one machine, and points at what is holding a run back.

## Now

Desktop monitoring for Linux: CPU, GPU, NPU, memory, processes, and network, with Ollama and vLLM awareness, tray, alerts, and themes.

## Next

- More local AI apps in the same workload view
- Throughput that is measured rather than estimated
- A history of inference and training sessions

## Later

- Advice on whether a model fits the current machine
- Container-aware process grouping
- A way to scrape this app's own metrics
- Deeper per-process accelerator attribution
