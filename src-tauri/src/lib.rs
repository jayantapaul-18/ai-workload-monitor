mod metrics;
mod settings;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use metrics::{
    get_hardware_profile, CollectOptions, HardwareProfile, MetricsEngine, MetricsSnapshot,
};
use settings::{AlertPayload, AppSettings, SettingsStore};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

fn app_icon() -> tauri::image::Image<'static> {
    tauri::include_image!("icons/32x32.png")
}

fn window_icon() -> tauri::image::Image<'static> {
    tauri::include_image!("icons/128x128.png")
}

struct AppState {
    engine: Mutex<MetricsEngine>,
    latest: Mutex<Option<MetricsSnapshot>>,
    settings: SettingsStore,
    last_ai_count: Mutex<u32>,
    alert_cooldowns: Mutex<std::collections::HashMap<String, Instant>>,
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> AppSettings {
    state.settings.get()
}

#[tauri::command]
fn save_settings(
    settings: AppSettings,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    state.settings.save(settings.clone())?;
    let _ = app.emit("settings-changed", &settings);
    Ok(())
}

#[tauri::command]
fn reset_settings(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let defaults = AppSettings::default();
    state.settings.save(defaults.clone())?;
    let _ = app.emit("settings-changed", &defaults);
    Ok(defaults)
}

#[tauri::command]
fn get_settings_path(state: tauri::State<'_, AppState>) -> String {
    state.settings.path_display()
}

#[tauri::command]
fn get_metrics(state: tauri::State<'_, AppState>) -> Option<MetricsSnapshot> {
    state.latest.lock().ok()?.clone()
}

#[tauri::command]
fn get_metrics_now(state: tauri::State<'_, AppState>) -> MetricsSnapshot {
    let settings = state.settings.get();
    let options = collect_options(&settings);
    let snapshot = state
        .engine
        .lock()
        .map(|mut e| e.collect(&options))
        .unwrap_or_else(|_| MetricsSnapshot::empty());
    if let Ok(mut latest) = state.latest.lock() {
        *latest = Some(snapshot.clone());
    }
    snapshot
}

#[tauri::command]
fn get_hardware_profile_cmd() -> HardwareProfile {
    get_hardware_profile()
}

#[tauri::command]
fn open_dashboard(app: tauri::AppHandle) {
    hide_tray_popup(&app);
    show_main_window(&app);
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn test_notification(app: tauri::AppHandle) -> Result<(), String> {
    send_desktop_notification(
        &app,
        "AI Workload Monitor",
        "Desktop notifications are working.",
    )
}

#[tauri::command]
fn export_metrics_snapshot(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let snapshot = state
        .latest
        .lock()
        .ok()
        .and_then(|s| s.clone())
        .ok_or_else(|| "No metrics available yet".to_string())?;
    serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())
}

fn collect_options(settings: &AppSettings) -> CollectOptions {
    CollectOptions {
        ai_keywords: settings.ai_keywords.clone(),
        process_limit: settings.process_limit,
        ai_intelligence_enabled: settings.ai_intelligence_enabled,
        ollama_url: settings.ollama_url.clone(),
        vllm_metrics_url: settings.vllm_metrics_url.clone(),
    }
}

fn poll_interval(settings: &AppSettings, window_focused: bool) -> Duration {
    let base = if settings.low_power_mode {
        settings.poll_interval_ms.max(2000)
    } else {
        settings.poll_interval_ms.clamp(250, 5000)
    };

    if settings.burst_poll_when_focused && window_focused && !settings.low_power_mode {
        Duration::from_millis((base / 2).max(250))
    } else {
        Duration::from_millis(base)
    }
}

fn start_polling(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        loop {
            let settings = state.settings.get();
            let focused = app
                .get_webview_window("main")
                .and_then(|w| w.is_focused().ok())
                .unwrap_or(false);

            let options = collect_options(&settings);
            let snapshot = {
                let Ok(mut engine) = state.engine.lock() else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                };
                engine.collect(&options)
            };

            if let Ok(mut latest) = state.latest.lock() {
                *latest = Some(snapshot.clone());
            }

            let _ = app.emit("metrics-update", &snapshot);
            check_alerts(&app, &state, &snapshot, &settings);
            update_tray_tooltip(&app, &snapshot, &settings);

            tokio::time::sleep(poll_interval(&settings, focused)).await;
        }
    });
}

fn check_alerts(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, AppState>,
    snapshot: &MetricsSnapshot,
    settings: &AppSettings,
) {
    if !settings.alerts_enabled {
        return;
    }

    let cooldown = Duration::from_secs(settings.alert_cooldown_secs.max(15));

    maybe_emit_alert(
        app,
        state,
        "cpu",
        snapshot.cpu.overall_usage,
        settings.cpu_alert_threshold,
        cooldown,
        "High CPU Usage",
        &format!("CPU at {:.0}%", snapshot.cpu.overall_usage),
    );

    maybe_emit_alert(
        app,
        state,
        "memory",
        snapshot.memory.usage_percent,
        settings.memory_alert_threshold,
        cooldown,
        "High Memory Usage",
        &format!("RAM at {:.0}%", snapshot.memory.usage_percent),
    );

    if let Some(temp) = snapshot.gpu.temperature_c {
        maybe_emit_alert(
            app,
            state,
            "gpu_temp",
            temp,
            settings.gpu_temp_alert_threshold,
            cooldown,
            "GPU Temperature",
            &format!("GPU at {:.0}°C", temp),
        );
    }

    if settings.notify_ai_process_changes {
        let count = snapshot.processes.ai_workloads.len() as u32;
        let Ok(mut last) = state.last_ai_count.lock() else {
            return;
        };
        if *last != count && *last > 0 {
            let payload = AlertPayload {
                kind: "ai_change".into(),
                title: "AI Workload Change".into(),
                message: format!("AI processes: {} → {}", *last, count),
                value: count as f32,
                threshold: *last as f32,
            };
            let _ = app.emit("alert-triggered", &payload);
            let _ = send_desktop_notification(app, &payload.title, &payload.message);
        }
        *last = count;
    }
}

#[allow(clippy::too_many_arguments)]
fn maybe_emit_alert(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, AppState>,
    key: &str,
    value: f32,
    threshold: f32,
    cooldown: Duration,
    title: &str,
    message: &str,
) {
    if value < threshold {
        return;
    }

    let Ok(mut cooldowns) = state.alert_cooldowns.lock() else {
        return;
    };
    if let Some(last) = cooldowns.get(key) {
        if last.elapsed() < cooldown {
            return;
        }
    }
    cooldowns.insert(key.to_string(), Instant::now());

    let payload = AlertPayload {
        kind: key.into(),
        title: title.into(),
        message: message.into(),
        value,
        threshold,
    };
    let _ = app.emit("alert-triggered", &payload);
    let _ = send_desktop_notification(app, title, message);
}

fn send_desktop_notification(
    app: &tauri::AppHandle,
    title: &str,
    body: &str,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

fn update_tray_tooltip(app: &tauri::AppHandle, snapshot: &MetricsSnapshot, settings: &AppSettings) {
    if !settings.show_tray_tooltip {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_tooltip(Some("AI Workload Monitor"));
        }
        return;
    }

    if let Some(tray) = app.tray_by_id("main") {
        let tooltip = format!(
            "CPU {:.0}% | GPU {:.0}% | RAM {:.0}% | ↓{} ↑{}",
            snapshot.cpu.overall_usage,
            snapshot.gpu.usage_percent,
            snapshot.memory.usage_percent,
            format_rate(snapshot.network.total_rx_bytes_per_sec),
            format_rate(snapshot.network.total_tx_bytes_per_sec),
        );
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

fn format_rate(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB/s", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0}KB/s", bytes as f64 / 1024.0)
    } else {
        format!("{bytes}B/s")
    }
}

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &settings_item,
            &hide,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let icon = app_icon();

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip("AI Workload Monitor")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "settings" => {
                show_main_window(app);
                let _ = app.emit("open-settings", ());
            }
            "hide" => hide_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                let app = tray.app_handle();
                let state = app.state::<AppState>();
                if state.settings.get().tray_popup_enabled {
                    toggle_tray_popup(app, Some(position));
                } else if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        show_main_window(app);
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn hide_tray_popup(app: &tauri::AppHandle) {
    if let Some(popup) = app.get_webview_window("tray-popup") {
        let _ = popup.hide();
    }
}

fn toggle_tray_popup(app: &tauri::AppHandle, position: Option<PhysicalPosition<f64>>) {
    let Some(popup) = app.get_webview_window("tray-popup") else {
        show_main_window(app);
        return;
    };

    if popup.is_visible().unwrap_or(false) {
        let _ = popup.hide();
        return;
    }

    if let Some(pos) = position {
        let x = (pos.x - 190.0).max(8.0);
        let y = (pos.y - 500.0).max(8.0);
        let _ = popup.set_position(PhysicalPosition::new(x, y));
    }

    let _ = popup.show();
    let _ = popup.set_focus();
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings_store = SettingsStore::new();
    let start_minimized = settings_store.get().start_minimized;

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            engine: Mutex::new(MetricsEngine::new()),
            latest: Mutex::new(None),
            settings: settings_store,
            last_ai_count: Mutex::new(0),
            alert_cooldowns: Mutex::new(std::collections::HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            reset_settings,
            get_settings_path,
            get_metrics,
            get_metrics_now,
            get_hardware_profile_cmd,
            open_dashboard,
            quit_app,
            test_notification,
            export_metrics_snapshot,
        ])
        .setup(move |app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(window_icon());
                if start_minimized {
                    let _ = window.hide();
                    if let Some(tray) = app.tray_by_id("main") {
                        let _ = tray.set_tooltip(Some(
                            "AI Workload Monitor — running in tray. Click icon for popup, or right-click → Show Dashboard.",
                        ));
                    }
                    let _ = send_desktop_notification(
                        app.handle(),
                        "AI Workload Monitor",
                        "Running in the system tray. Click the tray icon to open.",
                    );
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            build_tray(app.handle())?;
            start_polling(app.handle().clone());

            if app.state::<AppState>().settings.get().alerts_enabled {
                let _ = app.notification().request_permission();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "tray-popup" {
                if let WindowEvent::Focused(focused) = event {
                    if !focused {
                        let _ = window.hide();
                    }
                }
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if state.settings.get().close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}
