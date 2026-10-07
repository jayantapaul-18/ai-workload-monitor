use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::types::{InterfaceMetrics, NetworkMetrics, WifiMetrics};

pub struct NetworkCollector {
    prev_bytes: HashMap<String, (u64, u64)>,
    prev_sample: Instant,
    wifi_cache: Option<WifiMetrics>,
    wifi_last_fetch: Instant,
    ip_cache: HashMap<String, Vec<String>>,
    ip_last_fetch: Instant,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            prev_bytes: HashMap::new(),
            prev_sample: Instant::now(),
            wifi_cache: None,
            wifi_last_fetch: Instant::now() - std::time::Duration::from_secs(30),
            ip_cache: HashMap::new(),
            ip_last_fetch: Instant::now() - std::time::Duration::from_secs(30),
        }
    }

    pub fn collect(&mut self) -> NetworkMetrics {
        let elapsed = self.prev_sample.elapsed().as_secs_f64().max(0.001);
        let current = read_proc_net_dev();
        let mut interfaces = Vec::new();
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;

        if self.ip_last_fetch.elapsed().as_secs() >= 15 || self.ip_cache.is_empty() {
            self.ip_cache = read_all_ipv4();
            self.ip_last_fetch = Instant::now();
        }

        for (name, (rx, tx)) in &current {
            if name == "lo" {
                continue;
            }

            let (prev_rx, prev_tx) = self.prev_bytes.get(name).copied().unwrap_or((*rx, *tx));
            let rx_rate = ((*rx).saturating_sub(prev_rx) as f64 / elapsed) as u64;
            let tx_rate = ((*tx).saturating_sub(prev_tx) as f64 / elapsed) as u64;
            total_rx += rx_rate;
            total_tx += tx_rate;

            let ipv4 = self.ip_cache.get(name).cloned().unwrap_or_default();
            interfaces.push(build_interface(name, *rx, *tx, rx_rate, tx_rate, ipv4));
        }

        self.prev_bytes = current;
        self.prev_sample = Instant::now();

        interfaces.sort_by(|a, b| {
            (b.rx_bytes_per_sec + b.tx_bytes_per_sec)
                .cmp(&(a.rx_bytes_per_sec + a.tx_bytes_per_sec))
        });

        let wifi = self.collect_wifi(&interfaces);

        NetworkMetrics {
            interfaces,
            wifi,
            total_rx_bytes_per_sec: total_rx,
            total_tx_bytes_per_sec: total_tx,
        }
    }

    fn collect_wifi(&mut self, interfaces: &[InterfaceMetrics]) -> Option<WifiMetrics> {
        let wifi_iface = interfaces
            .iter()
            .find(|i| i.kind == "wifi")
            .map(|i| i.name.clone())?;

        if self.wifi_last_fetch.elapsed().as_secs() >= 10 {
            self.wifi_cache = Some(read_wifi_details(&wifi_iface));
            self.wifi_last_fetch = Instant::now();
        }

        let mut wifi = self.wifi_cache.clone().unwrap_or(WifiMetrics {
            interface: wifi_iface.clone(),
            ssid: String::new(),
            signal_dbm: None,
            signal_percent: None,
            frequency_mhz: None,
            bitrate_mbps: None,
            connected: false,
        });

        if let Some((_, level)) = read_wireless_signal(&wifi_iface) {
            wifi.signal_dbm = Some(level);
            wifi.signal_percent = Some(dbm_to_percent(level));
            wifi.connected = level > -90;
        }

        wifi.interface = wifi_iface;
        Some(wifi)
    }
}

fn build_interface(
    name: &str,
    rx_total: u64,
    tx_total: u64,
    rx_rate: u64,
    tx_rate: u64,
    ipv4: Vec<String>,
) -> InterfaceMetrics {
    let base = Path::new("/sys/class/net").join(name);
    let mac = read_file(base.join("address")).unwrap_or_default();
    let state = read_file(base.join("operstate")).unwrap_or_else(|| "unknown".into());
    let mtu = read_file(base.join("mtu"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let speed_mbps = read_file(base.join("speed"))
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|&s| s > 0)
        .map(|s| s as u64);

    InterfaceMetrics {
        name: name.to_string(),
        kind: classify_interface(name),
        mac,
        state,
        mtu,
        ipv4,
        rx_bytes_per_sec: rx_rate,
        tx_bytes_per_sec: tx_rate,
        rx_total_bytes: rx_total,
        tx_total_bytes: tx_total,
        speed_mbps,
    }
}

fn classify_interface(name: &str) -> String {
    if name.starts_with("wl") || name.starts_with("wlan") {
        "wifi".into()
    } else if name.starts_with("docker") || name.starts_with("br-") || name.starts_with("veth") {
        "virtual".into()
    } else if name.starts_with("en") || name.starts_with("eth") {
        "ethernet".into()
    } else {
        "other".into()
    }
}

fn read_proc_net_dev() -> HashMap<String, (u64, u64)> {
    let content = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let mut map = HashMap::new();

    for line in content.lines().skip(2) {
        let (iface, stats) = match line.split_once(':') {
            Some(v) => v,
            None => continue,
        };
        let iface = iface.trim().to_string();
        let parts: Vec<&str> = stats.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }
        let rx: u64 = parts[0].parse().unwrap_or(0);
        let tx: u64 = parts[8].parse().unwrap_or(0);
        map.insert(iface, (rx, tx));
    }

    map
}

fn read_all_ipv4() -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let Ok(output) = Command::new("ip")
        .args(["-4", "-o", "addr", "show"])
        .output()
    else {
        return map;
    };

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let iface = parts[1].to_string();
            let ip = parts[3].split('/').next().unwrap_or(parts[3]).to_string();
            map.entry(iface).or_default().push(ip);
        }
    }

    map
}

fn read_wireless_signal(iface: &str) -> Option<(String, i32)> {
    let content = fs::read_to_string("/proc/net/wireless").ok()?;
    for line in content.lines().skip(2) {
        if !line.contains(iface) {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let level = parts[3].replace('.', "").parse().ok()?;
            return Some((iface.to_string(), level));
        }
    }
    None
}

fn read_wifi_details(iface: &str) -> WifiMetrics {
    let mut wifi = WifiMetrics {
        interface: iface.to_string(),
        ssid: String::new(),
        signal_dbm: None,
        signal_percent: None,
        frequency_mhz: None,
        bitrate_mbps: None,
        connected: false,
    };

    if let Ok(output) = Command::new("iw").args(["dev", iface, "link"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if let Some(ssid) = line.strip_prefix("SSID: ") {
                wifi.ssid = ssid.to_string();
                wifi.connected = true;
            } else if let Some(signal) = line.strip_prefix("signal: ") {
                if let Some(dbm) = signal.split_whitespace().next() {
                    wifi.signal_dbm = dbm.parse().ok();
                    if let Some(d) = wifi.signal_dbm {
                        wifi.signal_percent = Some(dbm_to_percent(d));
                    }
                }
            } else if let Some(freq) = line.strip_prefix("freq: ") {
                wifi.frequency_mhz = freq.trim().parse().ok();
            } else if let Some(rate) = line.strip_prefix("tx bitrate: ") {
                if let Some(mbps) = rate.split_whitespace().next() {
                    wifi.bitrate_mbps = mbps.parse().ok();
                }
            }
        }
    }

    if wifi.ssid.is_empty() {
        if let Ok(output) = Command::new("nmcli")
            .args(["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let mut parts = line.split(':');
                if parts.next() == Some("yes") {
                    wifi.ssid = parts.next().unwrap_or("").to_string();
                    wifi.connected = !wifi.ssid.is_empty();
                    break;
                }
            }
        }
    }

    wifi
}

fn dbm_to_percent(dbm: i32) -> u32 {
    ((100.0 + dbm as f32) * 100.0 / 70.0).clamp(0.0, 100.0) as u32
}

fn read_file(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path.as_ref())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
