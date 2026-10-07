use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::types::{
    AiRuntimeMetrics, AiWorkloadEntry, BottleneckInsight, CpuMetrics, GpuMetrics, MemoryMetrics,
};

const HTTP_TIMEOUT: Duration = Duration::from_millis(800);
const SCRAPE_INTERVAL: Duration = Duration::from_millis(2000);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AiRuntimeConfig {
    enabled: bool,
    ollama_url: String,
    vllm_metrics_url: String,
    llama_cpp_url: String,
    comfyui_url: String,
}

#[derive(Clone, Default)]
struct ScrapedData {
    ollama_online: bool,
    ollama_models: Vec<OllamaModel>,
    ollama_cli: HashMap<String, OllamaCliRow>,
    vllm_online: bool,
    vllm_workloads: Vec<AiWorkloadEntry>,
    llama_cpp_online: bool,
    llama_cpp_workloads: Vec<AiWorkloadEntry>,
    comfyui_online: bool,
    comfyui_workloads: Vec<AiWorkloadEntry>,
}

struct AiScraperShared {
    config: Mutex<AiRuntimeConfig>,
    trigger: (Mutex<bool>, Condvar),
    scraped: Mutex<ScrapedData>,
    stop: AtomicBool,
    vllm_prev_tokens: Mutex<Option<(f64, Instant)>>,
}

pub struct AiRuntimeCollector {
    state: Arc<AiScraperShared>,
    _worker: Option<JoinHandle<()>>,
    ollama_tps_smooth: HashMap<String, f32>,
}

impl Default for AiRuntimeCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl AiRuntimeCollector {
    pub fn new() -> Self {
        let state = Arc::new(AiScraperShared {
            config: Mutex::new(AiRuntimeConfig::default()),
            trigger: (Mutex::new(false), Condvar::new()),
            scraped: Mutex::new(ScrapedData::default()),
            stop: AtomicBool::new(false),
            vllm_prev_tokens: Mutex::new(None),
        });

        let worker_state = Arc::clone(&state);
        let worker = thread::Builder::new()
            .name("ai-runtime-scraper".into())
            .spawn(move || {
                run_scraper_loop(worker_state);
            })
            .ok();

        Self {
            state,
            _worker: worker,
            ollama_tps_smooth: HashMap::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn collect(
        &mut self,
        enabled: bool,
        ollama_url: &str,
        vllm_metrics_url: &str,
        llama_cpp_url: &str,
        comfyui_url: &str,
        gpu: &GpuMetrics,
        cpu: &CpuMetrics,
        memory: &MemoryMetrics,
        runner_cpu: f32,
    ) -> AiRuntimeMetrics {
        if !enabled {
            return AiRuntimeMetrics::default();
        }

        // Check if config changed; if so, notify background scraper
        let mut config_changed = false;
        if let Ok(mut cfg) = self.state.config.lock() {
            if cfg.enabled != enabled
                || cfg.ollama_url != ollama_url
                || cfg.vllm_metrics_url != vllm_metrics_url
                || cfg.llama_cpp_url != llama_cpp_url
                || cfg.comfyui_url != comfyui_url
            {
                cfg.enabled = enabled;
                cfg.ollama_url = ollama_url.to_string();
                cfg.vllm_metrics_url = vllm_metrics_url.to_string();
                cfg.llama_cpp_url = llama_cpp_url.to_string();
                cfg.comfyui_url = comfyui_url.to_string();
                config_changed = true;
            }
        }
        if config_changed {
            self.state.trigger.1.notify_one();
        }

        // Sub-microsecond read of latest scraped runtime data from decoupled background cache
        let scraped = match self.state.scraped.lock() {
            Ok(data) => data.clone(),
            Err(_) => ScrapedData::default(),
        };

        let mut workloads = Vec::new();
        let mut runtimes_online = Vec::new();

        // 1. Ollama synthesis (correlating cached model list with live GPU & runner CPU)
        if scraped.ollama_online {
            runtimes_online.push("ollama".into());
            let gpu_inferencing = gpu.available && gpu.usage_percent > 25.0;
            let runner_active = runner_cpu > 20.0;

            let models = if scraped.ollama_models.is_empty() {
                scraped
                    .ollama_cli
                    .keys()
                    .map(|name| OllamaModel {
                        name: name.clone(),
                        size: 0,
                        size_vram: 0,
                        processor: scraped
                            .ollama_cli
                            .get(name)
                            .map(|r| r.processor.clone())
                            .unwrap_or_default(),
                        context_length: scraped.ollama_cli.get(name).and_then(|r| r.context_length),
                        details: None,
                    })
                    .collect()
            } else {
                scraped.ollama_models
            };

            for model in models {
                let cli = scraped.ollama_cli.get(&model.name);
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
                    &mut self.ollama_tps_smooth,
                );

                workloads.push(AiWorkloadEntry {
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
                });
            }
        }

        // 2. vLLM synthesis
        if scraped.vllm_online {
            runtimes_online.push("vllm".into());
            workloads.extend(scraped.vllm_workloads);
        }

        // 3. llama.cpp synthesis
        if scraped.llama_cpp_online {
            runtimes_online.push("llama_cpp".into());
            for mut entry in scraped.llama_cpp_workloads {
                // If GPU is active during llama.cpp processing, refine accelerator label
                if gpu.available && entry.status == "running" && gpu.usage_percent > 15.0 {
                    entry.accelerator = "GPU (active offload)".into();
                }
                workloads.push(entry);
            }
        }

        // 4. ComfyUI synthesis
        if scraped.comfyui_online {
            runtimes_online.push("comfyui".into());
            for mut entry in scraped.comfyui_workloads {
                if gpu.available && entry.status == "running" && gpu.usage_percent > 15.0 {
                    entry.accelerator = if !gpu.name.is_empty() {
                        gpu.name.clone()
                    } else {
                        "GPU (generating)".into()
                    };
                }
                workloads.push(entry);
            }
        }

        let bottleneck = diagnose_bottleneck(gpu, cpu, memory, &workloads);

        AiRuntimeMetrics {
            workloads,
            bottleneck,
            runtimes_online,
        }
    }
}

impl Drop for AiRuntimeCollector {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Relaxed);
        self.state.trigger.1.notify_all();
    }
}

fn run_scraper_loop(state: Arc<AiScraperShared>) {
    loop {
        if state.stop.load(Ordering::Relaxed) {
            break;
        }

        let config = match state.config.lock() {
            Ok(cfg) => cfg.clone(),
            Err(_) => break,
        };

        if config.enabled {
            let scraped = scrape_all(&config, &state.vllm_prev_tokens);
            if let Ok(mut cache) = state.scraped.lock() {
                *cache = scraped;
            }
        } else if let Ok(mut cache) = state.scraped.lock() {
            *cache = ScrapedData::default();
        }

        let (lock, cvar) = &state.trigger;
        if let Ok(guard) = lock.lock() {
            let _ = cvar.wait_timeout(guard, SCRAPE_INTERVAL);
        } else {
            thread::sleep(Duration::from_millis(500));
        }
    }
}

fn scrape_all(
    config: &AiRuntimeConfig,
    vllm_prev_tokens: &Mutex<Option<(f64, Instant)>>,
) -> ScrapedData {
    let mut data = ScrapedData::default();

    // 1. Ollama
    if !config.ollama_url.is_empty() {
        if let Some((models, cli_rows)) = scrape_ollama(&config.ollama_url) {
            data.ollama_online = true;
            data.ollama_models = models;
            data.ollama_cli = cli_rows;
        }
    }

    // 2. vLLM
    if !config.vllm_metrics_url.is_empty() {
        let prev = vllm_prev_tokens.lock().ok();
        if let Some(mut prev_guard) = prev {
            if let Some((online, workloads)) =
                scrape_vllm(&config.vllm_metrics_url, &mut prev_guard)
            {
                data.vllm_online = online;
                data.vllm_workloads = workloads;
            }
        }
    }

    // 3. llama.cpp (llama-server)
    if !config.llama_cpp_url.is_empty() {
        if let Some((online, workloads)) = scrape_llama_cpp(&config.llama_cpp_url) {
            data.llama_cpp_online = online;
            data.llama_cpp_workloads = workloads;
        }
    }

    // 4. ComfyUI
    if !config.comfyui_url.is_empty() {
        if let Some((online, workloads)) = scrape_comfyui(&config.comfyui_url) {
            data.comfyui_online = online;
            data.comfyui_workloads = workloads;
        }
    }

    data
}

// -----------------------------------------------------------------------------
// Ollama Scraping & Parsing
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct OllamaPsResponse {
    #[serde(default)]
    models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
struct OllamaModelDetails {
    #[serde(default)]
    parameter_size: String,
    #[serde(default)]
    quantization_level: String,
}

#[derive(Clone)]
struct OllamaCliRow {
    processor: String,
    context_length: Option<u32>,
}

fn scrape_ollama(base_url: &str) -> Option<(Vec<OllamaModel>, HashMap<String, OllamaCliRow>)> {
    let url = format!("{}/api/ps", base_url.trim_end_matches('/'));
    let response = ureq::get(&url).timeout(HTTP_TIMEOUT).call().ok()?;
    let body: OllamaPsResponse = response
        .into_string()
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())?;

    let cli_rows = parse_ollama_ps_cli(base_url);
    Some((body.models, cli_rows))
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

// -----------------------------------------------------------------------------
// vLLM Prometheus Scraping & Parsing
// -----------------------------------------------------------------------------

struct VllmScrape {
    online: bool,
    workloads: Vec<AiWorkloadEntry>,
}

fn scrape_vllm(
    metrics_url: &str,
    prev_tokens: &mut Option<(f64, Instant)>,
) -> Option<(bool, Vec<AiWorkloadEntry>)> {
    let response = ureq::get(metrics_url).timeout(HTTP_TIMEOUT).call().ok()?;
    let body = response.into_string().ok()?;
    let scrape = parse_vllm_prometheus(&body, prev_tokens);
    Some((scrape.online, scrape.workloads))
}

fn parse_vllm_prometheus(body: &str, prev_tokens: &mut Option<(f64, Instant)>) -> VllmScrape {
    let mut generation_tps: HashMap<String, f32> = HashMap::new();
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

// -----------------------------------------------------------------------------
// llama.cpp (llama-server) Scraping & Parsing
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct LlamaSlot {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    id_task: Option<i64>,
    #[serde(default)]
    n_ctx: Option<u32>,
    #[serde(default)]
    is_processing: bool,
    #[serde(default)]
    state: Option<i32>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlamaProps {
    #[serde(default)]
    default_generation_settings: Option<LlamaGenSettings>,
    #[serde(default)]
    model_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlamaGenSettings {
    #[serde(default)]
    n_ctx: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct LlamaHealth {
    #[serde(default)]
    status: String,
    #[serde(default)]
    slots_processing: Option<u32>,
}

fn scrape_llama_cpp(base_url: &str) -> Option<(bool, Vec<AiWorkloadEntry>)> {
    let base = base_url.trim_end_matches('/');

    // 1. Try querying /slots
    let slots_url = format!("{}/slots", base);
    let slots_resp = ureq::get(&slots_url).timeout(HTTP_TIMEOUT).call().ok();

    let mut slots = Vec::new();
    if let Some(resp) = slots_resp {
        if let Ok(text) = resp.into_string() {
            slots = parse_llama_cpp_slots(&text);
        }
    }

    // 2. Query /props for model metadata if needed
    let props_url = format!("{}/props", base);
    let props: Option<LlamaProps> = ureq::get(&props_url)
        .timeout(HTTP_TIMEOUT)
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .and_then(|t| serde_json::from_str(&t).ok());

    // 3. Fallback: query /health if both slots and props were empty
    let health: Option<LlamaHealth> = if slots.is_empty() && props.is_none() {
        let health_url = format!("{}/health", base);
        ureq::get(&health_url)
            .timeout(HTTP_TIMEOUT)
            .call()
            .ok()
            .and_then(|r| r.into_string().ok())
            .and_then(|t| serde_json::from_str(&t).ok())
    } else {
        None
    };

    if slots.is_empty() && props.is_none() && health.is_none() {
        return None;
    }

    let workloads = build_llama_workloads(&slots, props.as_ref(), health.as_ref());
    Some((true, workloads))
}

fn parse_llama_cpp_slots(text: &str) -> Vec<LlamaSlot> {
    if let Ok(slots) = serde_json::from_str::<Vec<LlamaSlot>>(text) {
        slots
    } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(text) {
        val.get("slots")
            .and_then(|s| serde_json::from_value::<Vec<LlamaSlot>>(s.clone()).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn build_llama_workloads(
    slots: &[LlamaSlot],
    props: Option<&LlamaProps>,
    health: Option<&LlamaHealth>,
) -> Vec<AiWorkloadEntry> {
    let raw_model = slots
        .iter()
        .find_map(|s| s.model.clone())
        .filter(|m| !m.is_empty())
        .or_else(|| props.and_then(|p| p.model_path.clone()))
        .unwrap_or_else(|| "llama-server".to_string());

    let (model_display, param_size, quant) = extract_model_details(&raw_model);

    let active_slots = slots
        .iter()
        .filter(|s| s.is_processing || s.state == Some(1))
        .count() as u32;

    let requests_running = if !slots.is_empty() {
        Some(active_slots)
    } else {
        health.and_then(|h| h.slots_processing)
    };

    let is_running = requests_running.map(|r| r > 0).unwrap_or(false);

    let context_length = slots
        .iter()
        .find_map(|s| s.n_ctx)
        .or_else(|| props.and_then(|p| p.default_generation_settings.as_ref()?.n_ctx));

    vec![AiWorkloadEntry {
        runtime: "llama_cpp".into(),
        model: model_display,
        status: if is_running {
            "running".into()
        } else {
            "loaded".into()
        },
        accelerator: "GPU (offload)".into(),
        vram_bytes: None,
        memory_on_gpu: true,
        tokens_per_sec: None,
        throughput_kind: if is_running {
            "active".into()
        } else {
            "none".into()
        },
        parameter_size: param_size,
        quantization: quant,
        context_length,
        requests_running,
    }]
}

fn is_param_size(s: &str) -> bool {
    let lower = s.to_lowercase();
    if (lower.ends_with('b') || lower.ends_with('m')) && lower.len() >= 2 {
        let num = &lower[..lower.len() - 1];
        if !num.is_empty()
            && num.chars().all(|c| c.is_ascii_digit() || c == '.')
            && num.chars().any(|c| c.is_ascii_digit())
        {
            return true;
        }
    }
    false
}

fn extract_model_details(path_or_name: &str) -> (String, Option<String>, Option<String>) {
    let filename = path_or_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path_or_name)
        .to_string();

    let mut param_size = None;
    for part in filename.split(['-', '_', ' ']) {
        let token = part.strip_suffix(".gguf").unwrap_or(part);
        if is_param_size(token) {
            param_size = Some(token.to_uppercase());
            break;
        }
        for sub in token.split('.') {
            if is_param_size(sub) {
                param_size = Some(sub.to_uppercase());
                break;
            }
        }
        if param_size.is_some() {
            break;
        }
    }

    let quants = [
        "Q4_K_M", "Q4_K_S", "Q4_0", "Q4_1", "Q5_K_M", "Q5_K_S", "Q5_0", "Q5_1", "Q6_K", "Q8_0",
        "Q8_1", "Q2_K", "Q3_K_M", "Q3_K_S", "Q3_K_L", "IQ4_NL", "IQ4_XS", "IQ3_M", "IQ3_S",
        "IQ2_XXS", "IQ2_XS", "BF16", "FP16", "F16", "FP32", "F32",
    ];

    let mut quant = None;
    let upper = filename.to_uppercase();
    for q in quants {
        if upper.contains(q) {
            quant = Some(q.to_string());
            break;
        }
    }

    (filename, param_size, quant)
}

// -----------------------------------------------------------------------------
// ComfyUI Scraping & Parsing
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
struct ComfyQueueResponse {
    #[serde(default)]
    queue_running: Vec<serde_json::Value>,
    #[serde(default)]
    queue_pending: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ComfySystemStats {
    #[serde(default)]
    devices: Vec<ComfyDevice>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ComfyDevice {
    #[serde(default)]
    name: String,
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    vram_total: Option<u64>,
    #[serde(default)]
    vram_free: Option<u64>,
    #[serde(default)]
    torch_vram_total: Option<u64>,
}

fn scrape_comfyui(base_url: &str) -> Option<(bool, Vec<AiWorkloadEntry>)> {
    let base = base_url.trim_end_matches('/');

    let queue_url = format!("{}/queue", base);
    let queue_resp = ureq::get(&queue_url).timeout(HTTP_TIMEOUT).call().ok()?;
    let queue_body = queue_resp.into_string().ok()?;
    let queue: ComfyQueueResponse = serde_json::from_str(&queue_body).ok()?;

    let stats_url = format!("{}/system_stats", base);
    let stats: Option<ComfySystemStats> = ureq::get(&stats_url)
        .timeout(HTTP_TIMEOUT)
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .and_then(|t| serde_json::from_str(&t).ok());

    let entry = parse_comfyui_data(&queue, stats.as_ref());
    Some((true, vec![entry]))
}

fn parse_comfyui_data(
    queue: &ComfyQueueResponse,
    stats: Option<&ComfySystemStats>,
) -> AiWorkloadEntry {
    let running_count = queue.queue_running.len() as u32;
    let is_running = running_count > 0;

    let model_name = extract_comfyui_model(&queue.queue_running)
        .unwrap_or_else(|| "ComfyUI Workflow".to_string());

    let (accelerator, vram_bytes, is_gpu) = if let Some(dev) = stats.and_then(|s| s.devices.first())
    {
        let is_cuda = dev.r#type == "cuda"
            || dev.name.to_lowercase().contains("nvidia")
            || dev.name.to_lowercase().contains("rtx")
            || dev.name.to_lowercase().contains("geforce");
        let is_mps = dev.r#type == "mps" || dev.name.to_lowercase().contains("apple");
        let name = if !dev.name.is_empty() {
            dev.name.clone()
        } else if is_cuda {
            "GPU (CUDA)".into()
        } else {
            "GPU".into()
        };
        let vram = dev
            .torch_vram_total
            .or_else(|| match (dev.vram_total, dev.vram_free) {
                (Some(tot), Some(free)) if tot >= free => Some(tot - free),
                _ => None,
            });
        (name, vram, is_cuda || is_mps)
    } else {
        ("GPU".into(), None, true)
    };

    AiWorkloadEntry {
        runtime: "comfyui".into(),
        model: model_name,
        status: if is_running {
            "running".into()
        } else {
            "idle".into()
        },
        accelerator,
        vram_bytes,
        memory_on_gpu: is_gpu,
        tokens_per_sec: None,
        throughput_kind: if is_running {
            "active".into()
        } else {
            "none".into()
        },
        parameter_size: None,
        quantization: None,
        context_length: None,
        requests_running: Some(running_count),
    }
}

fn extract_comfyui_model(queue_running: &[serde_json::Value]) -> Option<String> {
    for item in queue_running {
        if let Some(prompt_obj) = item.get(2).and_then(|v| v.as_object()) {
            for (_node_id, node_val) in prompt_obj {
                if let Some(inputs) = node_val.get("inputs").and_then(|v| v.as_object()) {
                    for key in ["ckpt_name", "unet_name", "model_name", "checkpoint"] {
                        if let Some(name) = inputs.get(key).and_then(|v| v.as_str()) {
                            if !name.is_empty() {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// -----------------------------------------------------------------------------
// Bottleneck Diagnosis
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

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
    fn parses_llama_cpp_slots_json() {
        let json = r#"[
            {
                "id": 0,
                "n_ctx": 4096,
                "is_processing": true,
                "state": 1,
                "model": "qwen2.5-coder-7b-instruct-q4_k_m.gguf"
            },
            {
                "id": 1,
                "n_ctx": 4096,
                "is_processing": false,
                "state": 0,
                "model": "qwen2.5-coder-7b-instruct-q4_k_m.gguf"
            }
        ]"#;
        let slots = parse_llama_cpp_slots(json);
        assert_eq!(slots.len(), 2);
        assert!(slots[0].is_processing);
        assert_eq!(slots[0].n_ctx, Some(4096));

        let workloads = build_llama_workloads(&slots, None, None);
        assert_eq!(workloads.len(), 1);
        let entry = &workloads[0];
        assert_eq!(entry.runtime, "llama_cpp");
        assert_eq!(entry.model, "qwen2.5-coder-7b-instruct-q4_k_m.gguf");
        assert_eq!(entry.parameter_size, Some("7B".into()));
        assert_eq!(entry.quantization, Some("Q4_K_M".into()));
        assert_eq!(entry.status, "running");
        assert_eq!(entry.requests_running, Some(1));
        assert_eq!(entry.context_length, Some(4096));
    }

    #[test]
    fn extracts_llama_model_metadata() {
        let (name, param, quant) =
            extract_model_details("/models/Meta-Llama-3.1-70B-Instruct-Q8_0.gguf");
        assert_eq!(name, "Meta-Llama-3.1-70B-Instruct-Q8_0.gguf");
        assert_eq!(param, Some("70B".into()));
        assert_eq!(quant, Some("Q8_0".into()));

        let (name2, param2, quant2) = extract_model_details("deepseek-r1-1.5b-q4_k_m.gguf");
        assert_eq!(name2, "deepseek-r1-1.5b-q4_k_m.gguf");
        assert_eq!(param2, Some("1.5B".into()));
        assert_eq!(quant2, Some("Q4_K_M".into()));
    }

    #[test]
    fn parses_comfyui_queue_and_stats() {
        let queue_json = r#"{
            "queue_running": [
                [
                    0,
                    "prompt-1",
                    {
                        "3": {
                            "class_type": "CheckpointLoaderSimple",
                            "inputs": { "ckpt_name": "v1-5-pruned-emaonly.safetensors" }
                        }
                    }
                ]
            ],
            "queue_pending": []
        }"#;
        let queue: ComfyQueueResponse = serde_json::from_str(queue_json).unwrap();
        let stats_json = r#"{
            "devices": [
                {
                    "name": "NVIDIA GeForce RTX 4090",
                    "type": "cuda",
                    "vram_total": 25757220864,
                    "vram_free": 15000000000,
                    "torch_vram_total": 10757220864
                }
            ]
        }"#;
        let stats: ComfySystemStats = serde_json::from_str(stats_json).unwrap();

        let entry = parse_comfyui_data(&queue, Some(&stats));
        assert_eq!(entry.runtime, "comfyui");
        assert_eq!(entry.model, "v1-5-pruned-emaonly.safetensors");
        assert_eq!(entry.status, "running");
        assert_eq!(entry.requests_running, Some(1));
        assert_eq!(entry.accelerator, "NVIDIA GeForce RTX 4090");
        assert_eq!(entry.vram_bytes, Some(10757220864));
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
