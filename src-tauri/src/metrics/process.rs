use std::cmp::Reverse;
use std::collections::HashSet;
use std::time::{Duration, Instant};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use super::types::{ProcessEntry, ProcessMetrics};

pub struct ProcessCollector {
    system: System,
    last_refresh: Instant,
}

impl ProcessCollector {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );

        Self {
            system,
            last_refresh: Instant::now() - Duration::from_secs(2),
        }
    }

    pub fn collect(&mut self, ai_keywords: &[String], limit: u32) -> ProcessMetrics {
        let limit = limit.clamp(5, 30) as usize;

        if self.last_refresh.elapsed() >= Duration::from_millis(1500) {
            self.system.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::everything(),
            );
            self.last_refresh = Instant::now();
        }

        let entries: Vec<ProcessEntry> = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let name = process.name().to_string_lossy().to_string();
                let cmd = process
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                let is_ai = is_ai_process(&name, &cmd, ai_keywords);

                ProcessEntry {
                    pid: pid.as_u32(),
                    name,
                    cmd: truncate_cmd(&cmd, 120),
                    cpu_percent: process.cpu_usage(),
                    memory_bytes: process.memory() * 1024,
                    status: format!("{:?}", process.status()),
                    is_ai,
                }
            })
            .collect();

        let mut top_cpu = entries.clone();
        top_cpu.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        top_cpu.truncate(limit);

        let mut top_memory = entries.clone();
        top_memory.sort_by_key(|entry| Reverse(entry.memory_bytes));
        top_memory.truncate(limit);

        let mut ai_workloads: Vec<ProcessEntry> =
            entries.iter().filter(|p| p.is_ai).cloned().collect();
        ai_workloads.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ai_workloads.truncate(limit.max(15));

        let top_cpu_pids: HashSet<u32> = top_cpu.iter().map(|p| p.pid).collect();
        let mut background: Vec<ProcessEntry> = entries
            .into_iter()
            .filter(|p| p.cpu_percent < 0.5 && !top_cpu_pids.contains(&p.pid))
            .collect();
        background.sort_by_key(|entry| Reverse(entry.memory_bytes));
        background.truncate(limit);

        ProcessMetrics {
            top_cpu,
            top_memory,
            background,
            ai_workloads,
        }
    }
}

fn is_ai_process(name: &str, cmd: &str, patterns: &[String]) -> bool {
    let haystack = format!("{} {}", name.to_lowercase(), cmd.to_lowercase());
    patterns
        .iter()
        .any(|pattern| haystack.contains(&pattern.to_lowercase()))
}

fn truncate_cmd(cmd: &str, max: usize) -> String {
    if cmd.len() <= max {
        cmd.to_string()
    } else {
        format!("{}…", &cmd[..max.saturating_sub(1)])
    }
}
