use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};

use super::types::{
    AiRuntimeMetrics, AiWorkloadEntry, BottleneckInsight, CpuMetrics, GpuMetrics, MemoryMetrics,
};

const HTTP_TIMEOUT: Duration = Duration::from_millis(800);

#[derive(Default)]
pub struct AiRuntimeCollector {
    vllm_prev_tokens: Option<(f64, Instant)>,
    ollama_tps_smooth: std::collections::HashMap<String, f32>,
}

impl AiRuntimeCollector {
    #[allow(clippy::too_many_arguments)]
    pub fn collect(
        &mut self,
        enabled: bool,
        ollama_url: &str,
        vllm_metrics_url: &str,
        gpu: &GpuMetrics,
        cpu: &CpuMetrics,
        memory: &MemoryMetrics,
        runner_cpu: f32,
    ) -> AiRuntimeMetrics {
        if !enabled {
            return AiRuntimeMetrics::default();
        }

        let mut workloads = Vec::new();
        let mut runtimes_online = Vec::new();

        if let Some(ollama) =
            collect_ollama(ollama_url, gpu, &mut self.ollama_tps_smooth, runner_cpu)
        {
            runtimes_online.push("ollama".into());
            workloads.extend(ollama);
        }

        if let Some((vllm_online, vllm_workloads)) =
            collect_vllm(vllm_metrics_url, &mut self.vllm_prev_tokens)
        {
            if vllm_online {
                runtimes_online.push("vllm".into());
            }
            workloads.extend(vllm_workloads);
        }

        let bottleneck = diagnose_bottleneck(gpu, cpu, memory, &workloads);

        AiRuntimeMetrics {
            workloads,
            bottleneck,
            runtimes_online,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OllamaPsResponse {
    #[serde(default)]
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    size_vram: u64,
    #[serde(default)]
    processor: String,
    #[serde(default)]
    context_length: Option<u32>,
    #[serde(default)]
    details: Option<OllamaModelDetails>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    #[serde(default)]
    parameter_size: String,
    #[serde(default)]
    quantization_level: String,
}

fn collect_ollama(
    base_url: &str,
    gpu: &GpuMetrics,
    tps_smooth: &mut HashMap<String, f32>,
    runner_cpu: f32,
) -> Option<Vec<AiWorkloadEntry>> {
    let url = format!("{}/api/ps", base_url.trim_end_matches('/'));
    let response = ureq::get(&url).timeout(HTTP_TIMEOUT).call().ok()?;
    let body: OllamaPsResponse = response
        .into_string()
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())?;

    let cli_rows = parse_ollama_ps_cli(base_url);

    if body.models.is_empty() && cli_rows.is_empty() {
        return Some(Vec::new());
    }

    let gpu_inferencing = gpu.available && gpu.usage_percent > 25.0;
    let runner_active = runner_cpu > 20.0;

    let models: Vec<OllamaModel> = if body.models.is_empty() {
        cli_rows
            .keys()
            .map(|name| OllamaModel {
                name: name.clone(),
                size: 0,
                size_vram: 0,
                processor: cli_rows
                    .get(name)
                    .map(|r| r.processor.clone())
                    .unwrap_or_default(),
                context_length: cli_rows.get(name).and_then(|r| r.context_length),
                details: None,
            })
            .collect()
    } else {
        body.models
    };

    Some(
        models
            .into_iter()
            .map(|model| {
                let cli = cli_rows.get(&model.name);
                let processor_field = if model.processor.is_empty() {
                    cli.map(|r| r.processor.as_str()).unwrap_or("")
                } else {
                    model.processor.as_str()
                };

                let parameter_size = model
                    .details
                    .as_ref()
                    .map(|d| d.parameter_size.clone())
                    .filter(|s| !s.is_empty());
                let quantization = model
                    .details
                    .as_ref()
                    .map(|d| d.quantization_level.clone())
                    .filter(|s| !s.is_empty());

                let inferencing = gpu_inferencing || (runner_active && cli.is_some());
                let accelerator =
                    infer_ollama_accelerator(processor_field, model.size_vram, gpu, inferencing);
                let (memory_bytes, memory_on_gpu) =
                    resolve_ollama_memory(model.size_vram, model.size, &accelerator);

                let (tokens_per_sec, throughput_kind) = resolve_ollama_throughput(
                    &model.name,
                    inferencing,
                    runner_cpu,
                    parameter_size.as_deref(),
                    quantization.as_deref(),
                    &accelerator,
                    tps_smooth,
                );

                AiWorkloadEntry {
                    runtime: "ollama".into(),
                    model: model.name,
                    status: if inferencing {
                        "running".into()
                    } else {
                        "loaded".into()
                    },
                    accelerator,
                    vram_bytes: memory_bytes,
                    memory_on_gpu,
                    tokens_per_sec,
                    throughput_kind,
                    parameter_size,
                    quantization,
                    context_length: model
                        .context_length
                        .or_else(|| cli.and_then(|r| r.context_length)),
                    requests_running: if inferencing { Some(1) } else { None },
                }
            })
            .collect(),
    )
}

struct OllamaCliRow {
    processor: String,
    context_length: Option<u32>,
}

fn parse_ollama_ps_cli(base_url: &str) -> HashMap<String, OllamaCliRow> {
    let mut rows = HashMap::new();
    let host = ollama_host_from_url(base_url);

    let output = match Command::new("ollama")
        .env("OLLAMA_HOST", host)
        .arg("ps")
        .output()
    {
        Ok(o) => o,
        Err(_) => return rows,
    };

    if !output.status.success() {
        return rows;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 7 {
            continue;
        }
        let name = parts[0].to_string();
        let processor = format!("{} {}", parts[4], parts[5]);
        let context_length = parts[6].parse().ok();
        rows.insert(
            name,
            OllamaCliRow {
                processor,
                context_length,
            },
        );
    }

    rows
}

fn ollama_host_from_url(base_url: &str) -> String {
    base_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .to_string()
}

fn resolve_ollama_throughput(
    model: &str,
    inferencing: bool,
    runner_cpu: f32,
    parameter_size: Option<&str>,
    quantization: Option<&str>,
    accelerator: &str,
    tps_smooth: &mut HashMap<String, f32>,
) -> (Option<f32>, String) {
    if !inferencing {
        return (None, "none".into());
    }

    if let Some(estimated) =
        estimate_ollama_tps(parameter_size, quantization, accelerator, runner_cpu)
    {
        let prev = tps_smooth.get(model).copied().unwrap_or(estimated);
        let smoothed = prev * 0.6 + estimated * 0.4;
        tps_smooth.insert(model.to_string(), smoothed);
        return (Some(smoothed), "estimated".into());
    }

    (None, "active".into())
}

fn estimate_ollama_tps(
    parameter_size: Option<&str>,
    quantization: Option<&str>,
    accelerator: &str,
    runner_cpu: f32,
) -> Option<f32> {
    if runner_cpu < 12.0 {
        return None;
    }

    let params_b: f32 = parameter_size
        .and_then(|p| p.trim().strip_suffix('B'))
        .and_then(|n| n.parse().ok())?;

    let on_gpu = accelerator.contains("GPU");
    let mut base = if on_gpu {
        params_b * 14.0
    } else {
        params_b * 7.0
    };

    if let Some(quant) = quantization {
        if quant.contains("Q8") {
            base *= 0.75;
        } else if quant.contains("Q2") {
            base *= 1.2;
        }
    }

    let scale = (runner_cpu / 100.0).clamp(0.25, 1.0);
    Some((base * scale).max(1.0))
}

fn infer_ollama_accelerator(
    processor: &str,
    size_vram: u64,
    gpu: &GpuMetrics,
    inferencing: bool,
) -> String {
    let p = processor.trim();
    if !p.is_empty() {
        return p.to_string();
    }

    if size_vram > 0 {
        return "GPU".into();
    }

    if gpu.available {
        if inferencing && gpu.usage_percent > 10.0 {
            return "GPU (active)".into();
        }
        if gpu.vram_used_bytes.unwrap_or(0) > 512 * 1024 * 1024 {
            return "GPU (inferred)".into();
        }
    }

    "CPU · system RAM".into()
}

fn resolve_ollama_memory(
    size_vram: u64,
    model_size: u64,
    accelerator: &str,
) -> (Option<u64>, bool) {
    if size_vram > 0 {
        return (Some(size_vram), true);
    }

    if model_size > 0 {
        let on_gpu = accelerator.contains("GPU");
        return (Some(model_size), on_gpu);
    }

    (None, false)
}

struct VllmScrape {
    online: bool,
    workloads: Vec<AiWorkloadEntry>,
}

fn collect_vllm(
    metrics_url: &str,
    prev_tokens: &mut Option<(f64, Instant)>,
) -> Option<(bool, Vec<AiWorkloadEntry>)> {
    let response = ureq::get(metrics_url).timeout(HTTP_TIMEOUT).call().ok()?;
    let body = response.into_string().ok()?;
    let scrape = parse_vllm_prometheus(&body, prev_tokens);
    Some((scrape.online, scrape.workloads))
}

fn parse_vllm_prometheus(body: &str, prev_tokens: &mut Option<(f64, Instant)>) -> VllmScrape {
    let mut generation_tps: std::collections::HashMap<String, f32> =
        std::collections::HashMap::new();
    let mut gpu_cache_usage: Option<f32> = None;
    let mut requests_running = 0u32;
    let mut total_generation_tokens = 0f64;

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("vllm:avg_generation_throughput_toks_per_s") {
            if let Some((model, value)) = parse_labeled_metric(line) {
                generation_tps.insert(model, value as f32);
            }
        } else if line.starts_with("vllm:gpu_cache_usage_perc") {
            gpu_cache_usage = parse_scalar_metric(line).map(|v| v as f32);
        } else if line.starts_with("vllm:num_requests_running") {
            requests_running = parse_scalar_metric(line).unwrap_or(0.0) as u32;
        } else if line.starts_with("vllm:generation_tokens_total") {
            if let Some(value) = parse_scalar_metric(line) {
                total_generation_tokens += value;
            }
        }
    }

    let now = Instant::now();
    let derived_tps = if let Some((prev_total, prev_time)) = prev_tokens {
        let elapsed = now.duration_since(*prev_time).as_secs_f64();
        if elapsed > 0.2 {
            Some(((total_generation_tokens - *prev_total) / elapsed) as f32)
        } else {
            None
        }
    } else {
        None
    };
    *prev_tokens = Some((total_generation_tokens, now));

    let mut workloads = Vec::new();

    if generation_tps.is_empty() && requests_running == 0 && gpu_cache_usage.is_none() {
        return VllmScrape {
            online: true,
            workloads,
        };
    }

    if generation_tps.is_empty() {
        let tps = derived_tps.or_else(|| generation_tps.values().next().copied());

        workloads.push(AiWorkloadEntry {
            runtime: "vllm".into(),
            model: "vLLM engine".into(),
            status: if requests_running > 0 {
                "running".into()
            } else {
                "idle".into()
            },
            accelerator: "GPU".into(),
            vram_bytes: gpu_cache_usage.map(|pct| (pct * 10_000_000.0) as u64),
            memory_on_gpu: true,
            tokens_per_sec: tps,
            throughput_kind: if tps.is_some() {
                "measured".into()
            } else {
                "none".into()
            },
            parameter_size: None,
            quantization: None,
            context_length: None,
            requests_running: Some(requests_running),
        });
    } else {
        for (model, tps) in generation_tps {
            workloads.push(AiWorkloadEntry {
                runtime: "vllm".into(),
                model,
                status: if requests_running > 0 || tps > 0.5 {
                    "running".into()
                } else {
                    "idle".into()
                },
                accelerator: "GPU".into(),
                vram_bytes: gpu_cache_usage.map(|pct| (pct * 10_000_000.0) as u64),
                memory_on_gpu: true,
                tokens_per_sec: Some(tps),
                throughput_kind: "measured".into(),
                parameter_size: None,
                quantization: None,
                context_length: None,
                requests_running: Some(requests_running),
            });
        }
    }

    VllmScrape {
        online: true,
        workloads,
    }
}

fn parse_labeled_metric(line: &str) -> Option<(String, f64)> {
    let model = line
        .split("model_name=\"")
        .nth(1)?
        .split('"')
        .next()?
        .to_string();
    let value = line.rsplit(' ').next()?.parse().ok()?;
    Some((model, value))
}

fn parse_scalar_metric(line: &str) -> Option<f64> {
    line.rsplit(' ').next()?.parse().ok()
}

fn diagnose_bottleneck(
    gpu: &GpuMetrics,
    cpu: &CpuMetrics,
    memory: &MemoryMetrics,
    workloads: &[AiWorkloadEntry],
) -> Option<BottleneckInsight> {
    if let (Some(used), Some(total)) = (gpu.vram_used_bytes, gpu.vram_total_bytes) {
        if total > 0 {
            let pct = (used as f32 / total as f32) * 100.0;
            if pct >= 92.0 {
                return Some(BottleneckInsight {
                    kind: "vram".into(),
                    severity: "high".into(),
                    message: format!("GPU VRAM at {:.0}%", pct),
                    suggestion:
                        "Use a smaller quant (Q4), reduce context length, or unload a model.".into(),
                });
            }
        }
    }

    if let Some(temp) = gpu.temperature_c {
        if temp >= 88.0 {
            return Some(BottleneckInsight {
                kind: "thermal".into(),
                severity: "high".into(),
                message: format!("GPU temperature {:.0}°C — possible throttling", temp),
                suggestion: "Improve cooling or reduce batch size / concurrent models.".into(),
            });
        }
    }

    if memory.usage_percent >= 92.0 {
        return Some(BottleneckInsight {
            kind: "memory".into(),
            severity: "high".into(),
            message: format!("System RAM at {:.0}%", memory.usage_percent),
            suggestion: "Close unused apps or use a smaller model quant.".into(),
        });
    }

    let has_loaded_models = workloads
        .iter()
        .any(|w| w.status == "loaded" || w.status == "running");
    if has_loaded_models && gpu.available && gpu.usage_percent < 15.0 && cpu.overall_usage > 60.0 {
        return Some(BottleneckInsight {
            kind: "cpu".into(),
            severity: "medium".into(),
            message: "AI workload appears CPU-bound (high CPU, low GPU)".into(),
            suggestion: "Ensure GPU offload is enabled or use a GPU-accelerated runtime.".into(),
        });
    }

    if has_loaded_models && gpu.available && gpu.usage_percent > 85.0 {
        let low_tps = workloads
            .iter()
            .all(|w| w.tokens_per_sec.unwrap_or(0.0) < 5.0);
        if low_tps {
            return Some(BottleneckInsight {
                kind: "gpu".into(),
                severity: "medium".into(),
                message: "GPU busy but throughput is low".into(),
                suggestion: "May be memory-bandwidth limited — try smaller batch or faster quant."
                    .into(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu(usage: f32, temp: Option<f32>, used: Option<u64>, total: Option<u64>) -> GpuMetrics {
        GpuMetrics {
            available: true,
            name: "test".into(),
            usage_percent: usage,
            temperature_c: temp,
            power_watts: None,
            frequency_mhz: None,
            vram_used_bytes: used,
            vram_total_bytes: total,
        }
    }

    fn cpu(usage: f32) -> CpuMetrics {
        CpuMetrics {
            cores: Vec::new(),
            overall_usage: usage,
            load_avg: [0.0, 0.0, 0.0],
            frequency_mhz: None,
        }
    }

    fn memory(percent: f32) -> MemoryMetrics {
        MemoryMetrics {
            total_bytes: 100,
            used_bytes: 0,
            available_bytes: 0,
            cached_bytes: 0,
            swap_total_bytes: 0,
            swap_used_bytes: 0,
            usage_percent: percent,
        }
    }

    fn workload(status: &str, tokens_per_sec: Option<f32>) -> AiWorkloadEntry {
        AiWorkloadEntry {
            runtime: "ollama".into(),
            model: "test".into(),
            status: status.into(),
            accelerator: "GPU".into(),
            vram_bytes: None,
            memory_on_gpu: true,
            tokens_per_sec,
            throughput_kind: "measured".into(),
            parameter_size: None,
            quantization: None,
            context_length: None,
            requests_running: None,
        }
    }

    #[test]
    fn parses_labeled_vllm_throughput() {
        let body = "\
# HELP vllm:avg_generation_throughput_toks_per_s Average generation throughput
vllm:avg_generation_throughput_toks_per_s{model_name=\"llama3\"} 42.5
vllm:num_requests_running 1
vllm:gpu_cache_usage_perc 0.4
";
        let mut prev = None;
        let scrape = parse_vllm_prometheus(body, &mut prev);
        assert!(scrape.online);
        assert_eq!(scrape.workloads.len(), 1);
        assert_eq!(scrape.workloads[0].model, "llama3");
        assert_eq!(scrape.workloads[0].tokens_per_sec, Some(42.5));
        assert_eq!(scrape.workloads[0].throughput_kind, "measured");
        assert_eq!(scrape.workloads[0].requests_running, Some(1));
        assert!(prev.is_some());
    }

    #[test]
    fn comment_only_prometheus_has_no_workloads() {
        let mut prev = None;
        let scrape = parse_vllm_prometheus("# just a comment\n", &mut prev);
        assert!(scrape.online);
        assert!(scrape.workloads.is_empty());
    }

    #[test]
    fn rejects_unlabeled_metric_lines() {
        assert!(parse_labeled_metric("vllm:avg_generation_throughput_toks_per_s 1.0").is_none());
    }

    #[test]
    fn vram_pressure_is_high_severity() {
        let insight = diagnose_bottleneck(
            &gpu(10.0, Some(40.0), Some(95), Some(100)),
            &cpu(10.0),
            &memory(10.0),
            &[],
        )
        .expect("vram bottleneck");
        assert_eq!(insight.kind, "vram");
        assert_eq!(insight.severity, "high");
    }

    #[test]
    fn healthy_load_has_no_bottleneck() {
        let insight = diagnose_bottleneck(
            &gpu(20.0, Some(50.0), Some(10), Some(100)),
            &cpu(10.0),
            &memory(20.0),
            &[workload("running", Some(40.0))],
        );
        assert!(insight.is_none());
    }
}
