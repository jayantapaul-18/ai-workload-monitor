mod ai_runtimes;
mod cpu;
mod gpu;
mod hardware;
mod memory;
mod network;
mod npu;
mod process;
mod system;
mod types;

use ai_runtimes::AiRuntimeCollector;
use cpu::{unix_timestamp, CpuCollector};
use gpu::GpuCollector;
pub use hardware::{get_hardware_profile, HardwareProfile};
use memory::MemoryCollector;
use network::NetworkCollector;
use npu::NpuCollector;
use process::ProcessCollector;
use system::SystemCollector;
pub use types::MetricsSnapshot;

#[derive(Clone, Default)]
pub struct CollectOptions {
    pub ai_keywords: Vec<String>,
    pub process_limit: u32,
    pub ai_intelligence_enabled: bool,
    pub ollama_url: String,
    pub vllm_metrics_url: String,
    pub llama_cpp_url: String,
    pub comfyui_url: String,
}

pub struct MetricsEngine {
    cpu: CpuCollector,
    memory: MemoryCollector,
    gpu: GpuCollector,
    npu: NpuCollector,
    process: ProcessCollector,
    network: NetworkCollector,
    system: SystemCollector,
    ai_runtimes: AiRuntimeCollector,
    primed: bool,
}

impl MetricsEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            cpu: CpuCollector::new(),
            memory: MemoryCollector::new(),
            gpu: GpuCollector::new(),
            npu: NpuCollector::new(),
            process: ProcessCollector::new(),
            network: NetworkCollector::new(),
            system: SystemCollector::new(),
            ai_runtimes: AiRuntimeCollector::default(),
            primed: false,
        };
        engine.prime();
        engine
    }

    fn prime(&mut self) {
        self.cpu.finalize_prev();
        let _ = self.network.collect();
        self.primed = true;
    }

    pub fn collect(&mut self, options: &CollectOptions) -> MetricsSnapshot {
        if !self.primed {
            self.prime();
        }

        let mut system = self.system.collect();
        system.uptime_secs = self.system.uptime_secs();

        let cpu = self.cpu.collect();
        let memory = self.memory.collect();
        let gpu = self.gpu.collect();
        let processes = self
            .process
            .collect(&options.ai_keywords, options.process_limit);
        let runner_cpu = self.process.ollama_runner_cpu();
        let ai_runtimes = self.ai_runtimes.collect(
            options.ai_intelligence_enabled,
            &options.ollama_url,
            &options.vllm_metrics_url,
            &options.llama_cpp_url,
            &options.comfyui_url,
            &gpu,
            &cpu,
            &memory,
            runner_cpu,
        );

        MetricsSnapshot {
            timestamp: unix_timestamp(),
            cpu,
            memory,
            gpu,
            npu: self.npu.collect(),
            processes,
            network: self.network.collect(),
            system,
            ai_runtimes,
        }
    }
}
