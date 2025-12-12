use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub uptime_seconds: u64,
}

pub fn get_system_resources(sys: &mut System) -> SystemResources {
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    
    // Calculate average CPU usage across all CPUs
    let cpus = sys.cpus();
    let cpu_percent = if !cpus.is_empty() {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    } else {
        0.0
    };
    
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let memory_percent = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        0.0
    };
    let uptime_seconds = System::uptime();
    
    SystemResources {
        cpu_percent,
        memory_percent,
        memory_used,
        memory_total,
        uptime_seconds,
    }
}
