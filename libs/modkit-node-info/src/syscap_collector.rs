use crate::error::NodeInfoError;
use crate::model::*;
use crate::sysinfo_collector::SysInfoCollector;
use std::sync::Arc;

/// Collects system capabilities for the current node
pub struct SysCapCollector {
    sysinfo_collector: Arc<SysInfoCollector>,
}

impl SysCapCollector {
    pub fn new() -> Self {
        Self {
            sysinfo_collector: Arc::new(SysInfoCollector::new()),
        }
    }

    /// Helper to create a SysCap with cache metadata
    fn create_syscap(
        key: String,
        category: String,
        name: String,
        display_name: String,
        present: bool,
        version: Option<String>,
        amount: Option<f64>,
        amount_dimension: Option<String>,
        details: Option<String>,
        cache_ttl_secs: u64,
    ) -> SysCap {
        SysCap {
            key,
            category,
            name,
            display_name,
            present,
            version,
            amount,
            amount_dimension,
            details,
            cache_ttl_secs,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        }
    }

    /// Collect current system capabilities using sysinfo data
    pub fn collect(&self, node_id: uuid::Uuid) -> Result<NodeSysCap, NodeInfoError> {
        // Collect sysinfo first to use its data
        let sysinfo = self.sysinfo_collector.collect(node_id)?;

        let mut capabilities = Vec::new();

        // Collect hardware capabilities using sysinfo data
        capabilities.extend(self.collect_hardware_caps(&sysinfo)?);

        // Collect OS capabilities using sysinfo data
        capabilities.extend(self.collect_os_caps(&sysinfo)?);

        // Collect GPU capabilities using sysinfo data
        capabilities.extend(self.collect_gpu_caps(&sysinfo)?);

        // Collect battery capabilities using sysinfo data
        capabilities.extend(self.collect_battery_caps(&sysinfo)?);

        // Collect software capabilities
        capabilities.extend(self.collect_software_caps()?);

        Ok(NodeSysCap {
            node_id,
            capabilities,
            collected_at: chrono::Utc::now(),
        })
    }

    fn collect_hardware_caps(&self, sysinfo: &NodeSysInfo) -> Result<Vec<SysCap>, NodeInfoError> {
        let mut caps = Vec::new();

        // Architecture detection from sysinfo
        let arch = &sysinfo.os.arch;
        caps.push(Self::create_syscap(
            format!("hardware:{}", arch),
            "hardware".to_string(),
            arch.to_string(),
            arch.to_uppercase(),
            true,
            None,
            None,
            None,
            Some(format!("{} architecture detected", arch)),
            3600, // 1 hour cache (never changes)
        ));

        // RAM detection from sysinfo
        let total_gb = sysinfo.memory.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        caps.push(Self::create_syscap(
            "hardware:ram".to_string(),
            "hardware".to_string(),
            "ram".to_string(),
            "RAM".to_string(),
            true,
            None,
            Some(total_gb),
            Some("GB".to_string()),
            Some(format!(
                "Total: {:.2} GB, Used: {}%",
                total_gb, sysinfo.memory.used_percent
            )),
            5, // 5 seconds cache (changes frequently)
        ));

        // CPU capability from sysinfo
        caps.push(Self::create_syscap(
            "hardware:cpu".to_string(),
            "hardware".to_string(),
            "cpu".to_string(),
            "CPU".to_string(),
            true,
            Some(sysinfo.cpu.model.clone()),
            Some(sysinfo.cpu.cores as f64),
            Some("cores".to_string()),
            Some(format!(
                "{} with {} cores @ {:.0} MHz",
                sysinfo.cpu.model, sysinfo.cpu.cores, sysinfo.cpu.frequency_mhz
            )),
            600, // 10 minutes cache (changes rarely)
        ));

        Ok(caps)
    }

    fn collect_os_caps(&self, sysinfo: &NodeSysInfo) -> Result<Vec<SysCap>, NodeInfoError> {
        let mut caps = Vec::new();

        let os = std::env::consts::OS;
        caps.push(Self::create_syscap(
            format!("os:{}", os),
            "os".to_string(),
            os.to_string(),
            match os {
                "macos" => "macOS",
                "linux" => "Linux",
                "windows" => "Windows",
                _ => os,
            }
            .to_string(),
            true,
            Some(sysinfo.os.version.clone()),
            None,
            None,
            Some(format!(
                "Platform: {}, Version: {}, Arch: {}",
                sysinfo.os.name, sysinfo.os.version, sysinfo.os.arch
            )),
            120, // 2 minutes cache (OS doesn't change much)
        ));

        Ok(caps)
    }

    fn collect_gpu_caps(&self, sysinfo: &NodeSysInfo) -> Result<Vec<SysCap>, NodeInfoError> {
        let mut caps = Vec::new();

        for (i, gpu) in sysinfo.gpus.iter().enumerate() {
            let gpu_key = if i == 0 {
                "hardware:gpu".to_string()
            } else {
                format!("hardware:gpu{}", i)
            };

            let mut details = format!("Model: {}", gpu.model);
            if let Some(vram) = gpu.total_memory_mb {
                details.push_str(&format!(", VRAM: {:.0} MB", vram));
            }
            if let Some(cores) = gpu.cores {
                details.push_str(&format!(", Cores: {}", cores));
            }

            caps.push(Self::create_syscap(
                gpu_key,
                "hardware".to_string(),
                format!(
                    "gpu{}",
                    if i == 0 {
                        "".to_string()
                    } else {
                        i.to_string()
                    }
                ),
                "GPU".to_string(),
                true,
                Some(gpu.model.clone()),
                gpu.total_memory_mb,
                if gpu.total_memory_mb.is_some() {
                    Some("MB".to_string())
                } else {
                    None
                },
                Some(details),
                10, // 10 seconds cache (changes frequently)
            ));
        }

        Ok(caps)
    }

    fn collect_battery_caps(&self, sysinfo: &NodeSysInfo) -> Result<Vec<SysCap>, NodeInfoError> {
        let mut caps = Vec::new();

        if let Some(battery) = &sysinfo.battery {
            let status = if battery.on_battery {
                "discharging (on battery power)"
            } else {
                "charging"
            };

            caps.push(Self::create_syscap(
                "hardware:battery".to_string(),
                "hardware".to_string(),
                "battery".to_string(),
                "Battery".to_string(),
                true,
                None,
                Some(battery.percentage as f64),
                Some("percent".to_string()),
                Some(format!(
                    "Status: {}, Level: {}%",
                    status, battery.percentage
                )),
                3, // 3 seconds cache (battery changes frequently)
            ));
        }

        Ok(caps)
    }

    fn collect_software_caps(&self) -> Result<Vec<SysCap>, NodeInfoError> {
        // Software capability detection can be extended here
        // For now, return empty list
        Ok(Vec::new())
    }
}

impl Default for SysCapCollector {
    fn default() -> Self {
        Self::new()
    }
}
