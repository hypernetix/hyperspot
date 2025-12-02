use crate::error::NodeInfoError;
use crate::model::*;
use sysinfo::System;

/// Collects system information for the current node
pub struct SysInfoCollector {
    system: std::sync::Mutex<System>,
}

impl SysInfoCollector {
    pub fn new() -> Self {
        let system = System::new_all();
        Self {
            system: std::sync::Mutex::new(system),
        }
    }

    /// Collect current system information
    pub fn collect(&self, node_id: uuid::Uuid) -> Result<NodeSysInfo, NodeInfoError> {
        let mut sys = self
            .system
            .lock()
            .map_err(|e| NodeInfoError::SysInfoCollectionFailed(e.to_string()))?;

        // Refresh system information
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let os_info = self.collect_os_info(&sys)?;
        let cpu_info = self.collect_cpu_info(&sys)?;
        let memory_info = self.collect_memory_info(&sys)?;
        let host_info = self.collect_host_info()?;
        let gpus = self.collect_gpu_info()?;
        let battery = self.collect_battery_info();

        Ok(NodeSysInfo {
            node_id,
            os: os_info,
            cpu: cpu_info,
            memory: memory_info,
            host: host_info,
            gpus,
            battery,
            collected_at: chrono::Utc::now(),
        })
    }

    fn collect_os_info(&self, _sys: &System) -> Result<OsInfo, NodeInfoError> {
        let name = System::name().unwrap_or_else(|| std::env::consts::OS.to_string());
        let version = System::os_version().unwrap_or_else(|| "unknown".to_string());
        let arch = std::env::consts::ARCH.to_string();

        Ok(OsInfo {
            name,
            version,
            arch,
        })
    }

    fn collect_cpu_info(&self, sys: &System) -> Result<CpuInfo, NodeInfoError> {
        let cpus = sys.cpus();
        let num_cpus = cpus.len() as u32;

        let model = if let Some(cpu) = cpus.first() {
            cpu.brand().to_string()
        } else {
            "Unknown".to_string()
        };

        // Get physical core count
        let cores = sys.physical_core_count().unwrap_or(num_cpus as usize) as u32;

        // Get average frequency
        let frequency_mhz = if !cpus.is_empty() {
            cpus.iter().map(|cpu| cpu.frequency() as f64).sum::<f64>() / cpus.len() as f64
        } else {
            0.0
        };

        Ok(CpuInfo {
            model,
            num_cpus,
            cores,
            frequency_mhz,
        })
    }

    fn collect_memory_info(&self, sys: &System) -> Result<MemoryInfo, NodeInfoError> {
        let total_bytes = sys.total_memory();
        let available_bytes = sys.available_memory();
        let used_bytes = sys.used_memory();
        let used_percent = if total_bytes > 0 {
            ((used_bytes as f64 / total_bytes as f64) * 100.0) as u32
        } else {
            0
        };

        Ok(MemoryInfo {
            total_bytes,
            available_bytes,
            used_bytes,
            used_percent,
        })
    }

    fn collect_host_info(&self) -> Result<HostInfo, NodeInfoError> {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let uptime_seconds = System::uptime();

        // Collect all IP addresses
        let mut ip_addresses = Vec::new();

        // First, add the primary IP (default route interface)
        if let Ok(primary_ip) = local_ip_address::local_ip() {
            ip_addresses.push(primary_ip.to_string());
        }

        // Then add all other network interface IPs
        if let Ok(all_ips) = local_ip_address::list_afinet_netifas() {
            for (_name, ip) in all_ips {
                let ip_str = ip.to_string();
                // Skip if already added as primary
                if !ip_addresses.contains(&ip_str) {
                    // Skip loopback addresses
                    if !ip.is_loopback() {
                        ip_addresses.push(ip_str);
                    }
                }
            }
        }

        Ok(HostInfo {
            hostname,
            uptime_seconds,
            ip_addresses,
        })
    }

    fn collect_gpu_info(&self) -> Result<Vec<GpuInfo>, NodeInfoError> {
        // Use platform-specific GPU detection
        #[cfg(target_os = "macos")]
        {
            super::gpu_collector_macos::collect_gpu_info()
        }
        #[cfg(target_os = "linux")]
        {
            super::gpu_collector_linux::collect_gpu_info()
        }
        #[cfg(target_os = "windows")]
        {
            super::gpu_collector_windows::collect_gpu_info()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Ok(Vec::new())
        }
    }

    fn collect_battery_info(&self) -> Option<BatteryInfo> {
        // Use starship-battery for cross-platform battery detection
        use starship_battery::Manager;

        let manager = Manager::new().ok()?;
        let mut batteries = manager.batteries().ok()?;

        if let Some(Ok(battery)) = batteries.next() {
            use starship_battery::State;

            let on_battery = matches!(battery.state(), State::Discharging);
            let percentage = (battery.state_of_charge().value * 100.0) as u32;

            Some(BatteryInfo {
                on_battery,
                percentage,
            })
        } else {
            // No battery detected (desktop system)
            None
        }
    }
}

impl Default for SysInfoCollector {
    fn default() -> Self {
        Self::new()
    }
}
