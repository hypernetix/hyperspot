use crate::error::NodeInfoError;
use crate::model::{BatteryInfo, CpuInfo, GpuInfo, HostInfo, MemoryInfo, NodeSysInfo, OsInfo};
use sysinfo::System;

/// Calculate percentage as u32 (0-100) from used/total values.
/// Returns 0 if total is 0 to avoid division by zero.
fn calculate_percent(used: u64, total: u64) -> u32 {
    if total == 0 {
        return 0;
    }
    // Use f64 for division to satisfy clippy::integer_division
    let percent = (used as f64 / total as f64) * 100.0;
    // Clamp to 0-100 range - percent is always positive here
    (percent.clamp(0.0, 100.0)) as u32
}

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

        let os_info = Self::collect_os_info();
        let cpu_info = Self::collect_cpu_info(&sys);
        let memory_info = Self::collect_memory_info(&sys);
        let host_info = Self::collect_host_info();
        let gpus = Self::collect_gpu_info();
        let battery = Self::collect_battery_info();

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

    fn collect_os_info() -> OsInfo {
        let name = System::name().unwrap_or_else(|| std::env::consts::OS.to_string());
        let version = System::os_version().unwrap_or_else(|| "unknown".to_string());
        let arch = std::env::consts::ARCH.to_string();

        OsInfo {
            name,
            version,
            arch,
        }
    }

    fn collect_cpu_info(sys: &System) -> CpuInfo {
        let cpus = sys.cpus();
        // CPU count is always small, safe to truncate
        let num_cpus = u32::try_from(cpus.len()).unwrap_or(u32::MAX);

        let model = if let Some(cpu) = cpus.first() {
            cpu.brand().to_string()
        } else {
            "Unknown".to_string()
        };

        // Get physical core count - always small, safe to truncate
        let cores =
            u32::try_from(System::physical_core_count().unwrap_or(cpus.len())).unwrap_or(u32::MAX);

        // Get average frequency
        let frequency_mhz = if !cpus.is_empty() {
            cpus.iter().map(|cpu| cpu.frequency() as f64).sum::<f64>() / cpus.len() as f64
        } else {
            0.0
        };

        CpuInfo {
            model,
            num_cpus,
            cores,
            frequency_mhz,
        }
    }

    fn collect_memory_info(sys: &System) -> MemoryInfo {
        let total_bytes = sys.total_memory();
        let available_bytes = sys.available_memory();
        let used_bytes = sys.used_memory();
        // Calculate percentage (0-100) using integer math to avoid float precision issues
        let used_percent = calculate_percent(used_bytes, total_bytes);

        MemoryInfo {
            total_bytes,
            available_bytes,
            used_bytes,
            used_percent,
        }
    }

    fn collect_host_info() -> HostInfo {
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
                // Skip if already added as primary and avoid loopback addresses
                if !ip_addresses.contains(&ip_str) && !ip.is_loopback() {
                    ip_addresses.push(ip_str);
                }
            }
        }

        HostInfo {
            hostname,
            uptime_seconds,
            ip_addresses,
        }
    }

    fn collect_gpu_info() -> Vec<GpuInfo> {
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
            Vec::new()
        }
    }

    fn collect_battery_info() -> Option<BatteryInfo> {
        // Use starship-battery for cross-platform battery detection
        use starship_battery::Manager;

        let manager = Manager::new().ok()?;
        let mut batteries = manager.batteries().ok()?;

        if let Some(Ok(battery)) = batteries.next() {
            use starship_battery::State;

            let on_battery = matches!(battery.state(), State::Discharging);
            // Battery percentage is 0.0-1.0, multiply by 100 and clamp to valid range
            let charge = f64::from(battery.state_of_charge().value) * 100.0;
            let percentage = charge.clamp(0.0, 100.0) as u32;

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
