use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // General
    pub start_minimized: bool,
    pub close_to_tray: bool,
    pub launch_at_login: bool,
    pub show_tray_tooltip: bool,
    #[serde(default = "default_tray_popup_enabled")]
    pub tray_popup_enabled: bool,

    // Monitoring
    pub poll_interval_ms: u64,
    pub burst_poll_when_focused: bool,
    pub process_limit: u32,
    pub history_length: u32,

    // Alerts
    pub alerts_enabled: bool,
    pub cpu_alert_threshold: f32,
    pub memory_alert_threshold: f32,
    pub gpu_temp_alert_threshold: f32,
    pub alert_cooldown_secs: u64,
    pub notify_ai_process_changes: bool,

    // AI Workloads
    pub ai_keywords: Vec<String>,
    pub highlight_ai_in_processes: bool,
    pub auto_focus_ai_tab: bool,
    #[serde(default = "default_ai_intelligence_enabled")]
    pub ai_intelligence_enabled: bool,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default = "default_vllm_metrics_url")]
    pub vllm_metrics_url: String,
    #[serde(default = "default_llama_cpp_url")]
    pub llama_cpp_url: String,
    #[serde(default = "default_comfyui_url")]
    pub comfyui_url: String,

    // Appearance
    #[serde(default = "default_color_theme")]
    pub color_theme: String,
    pub accent_theme: String,
    #[serde(default)]
    pub compact_sidebar: bool,
    pub compact_density: bool,
    pub show_sparklines: bool,
    pub animate_gauges: bool,

    // Advanced
    pub export_include_processes: bool,
    pub low_power_mode: bool,
}

fn default_color_theme() -> String {
    "dark".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_minimized: false,
            close_to_tray: true,
            launch_at_login: false,
            show_tray_tooltip: true,
            tray_popup_enabled: default_tray_popup_enabled(),
            poll_interval_ms: 1000,
            burst_poll_when_focused: true,
            process_limit: 12,
            history_length: 60,
            alerts_enabled: true,
            cpu_alert_threshold: 90.0,
            memory_alert_threshold: 90.0,
            gpu_temp_alert_threshold: 85.0,
            alert_cooldown_secs: 60,
            notify_ai_process_changes: false,
            ai_keywords: default_ai_keywords(),
            highlight_ai_in_processes: true,
            auto_focus_ai_tab: false,
            ai_intelligence_enabled: default_ai_intelligence_enabled(),
            ollama_url: default_ollama_url(),
            vllm_metrics_url: default_vllm_metrics_url(),
            llama_cpp_url: default_llama_cpp_url(),
            comfyui_url: default_comfyui_url(),
            color_theme: default_color_theme(),
            accent_theme: "green".into(),
            compact_sidebar: false,
            compact_density: false,
            show_sparklines: true,
            animate_gauges: true,
            export_include_processes: true,
            low_power_mode: false,
        }
    }
}

fn default_tray_popup_enabled() -> bool {
    true
}

fn default_ai_intelligence_enabled() -> bool {
    true
}

fn default_ollama_url() -> String {
    "http://127.0.0.1:11434".into()
}

fn default_vllm_metrics_url() -> String {
    "http://127.0.0.1:8000/metrics".into()
}

fn default_llama_cpp_url() -> String {
    "http://127.0.0.1:8080".into()
}

fn default_comfyui_url() -> String {
    "http://127.0.0.1:8188".into()
}

pub fn default_ai_keywords() -> Vec<String> {
    vec![
        "python".into(),
        "ollama".into(),
        "llama".into(),
        "vllm".into(),
        "torch".into(),
        "docker".into(),
        "containerd".into(),
        "jupyter".into(),
        "whisper".into(),
        "transformers".into(),
        "rocm".into(),
        "triton".into(),
        "inference".into(),
        "sglang".into(),
        "text-generation".into(),
        "llama.cpp".into(),
        "comfyui".into(),
        "stable-diffusion".into(),
        "cuda".into(),
        "fastapi".into(),
        "uvicorn".into(),
        "node".into(),
        "xinference".into(),
        "open-webui".into(),
    ]
}

pub struct SettingsStore {
    path: PathBuf,
    pub settings: Mutex<AppSettings>,
}

impl SettingsStore {
    pub fn new() -> Self {
        let path = settings_path();
        let settings = load_from_disk(&path).unwrap_or_default();
        Self {
            path,
            settings: Mutex::new(settings),
        }
    }

    pub fn get(&self) -> AppSettings {
        self.settings.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn save(&self, settings: AppSettings) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
        fs::write(&self.path, json).map_err(|e| e.to_string())?;
        if let Ok(mut guard) = self.settings.lock() {
            *guard = settings;
        }
        Ok(())
    }

    pub fn path_display(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai-workload-monitor")
        .join("settings.json")
}

fn load_from_disk(path: &PathBuf) -> Option<AppSettings> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertPayload {
    pub kind: String,
    pub title: String,
    pub message: String,
    pub value: f32,
    pub threshold: f32,
}
