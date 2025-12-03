use crate::error::NodeInfoError;
use crate::model::*;
use crate::sysinfo_collector::SysInfoCollector;
use std::sync::Arc;

/// Builder for creating SysCap instances with reduced parameter count
struct SysCapBuilder {
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
}

impl SysCapBuilder {
    fn new(key: String, category: String, name: String, display_name: String) -> Self {
        Self {
            key,
            category,
            name,
            display_name,
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs: 300, // Default 5 minutes
        }
    }

    fn version(mut self, version: Option<String>) -> Self {
        self.version = version;
        self
    }

    fn amount(mut self, amount: Option<f64>) -> Self {
        self.amount = amount;
        self
    }

    fn amount_dimension(mut self, amount_dimension: Option<String>) -> Self {
        self.amount_dimension = amount_dimension;
        self
    }

    fn details(mut self, details: Option<String>) -> Self {
        self.details = details;
        self
    }

    fn cache_ttl_secs(mut self, cache_ttl_secs: u64) -> Self {
        self.cache_ttl_secs = cache_ttl_secs;
        self
    }

    fn build(self) -> SysCap {
        SysCap {
            key: self.key,
            category: self.category,
            name: self.name,
            display_name: self.display_name,
            present: self.present,
            version: self.version,
            amount: self.amount,
            amount_dimension: self.amount_dimension,
            details: self.details,
            cache_ttl_secs: self.cache_ttl_secs,
            fetched_at_secs: chrono::Utc::now().timestamp(),
        }
    }
}

/// Collects system capabilities for the current node
pub struct SysCapCollector {
    sysinfo_collector: Arc<SysInfoCollector>,
}

impl SysCapCollector {
    /// Create a new SysCapCollector with a shared SysInfoCollector reference
    pub fn new(sysinfo_collector: Arc<SysInfoCollector>) -> Self {
        Self { sysinfo_collector }
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
        caps.push(
            SysCapBuilder::new(
                format!("hardware:{}", arch),
                "hardware".to_string(),
                arch.to_string(),
                arch.to_uppercase(),
            )
            .details(Some(format!("{} architecture detected", arch)))
            .cache_ttl_secs(3600) // 1 hour cache (never changes)
            .build(),
        );

        // RAM detection from sysinfo
        let total_gb = sysinfo.memory.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        caps.push(
            SysCapBuilder::new(
                "hardware:ram".to_string(),
                "hardware".to_string(),
                "ram".to_string(),
                "RAM".to_string(),
            )
            .amount(Some(total_gb))
            .amount_dimension(Some("GB".to_string()))
            .details(Some(format!(
                "Total: {:.2} GB, Used: {}%",
                total_gb, sysinfo.memory.used_percent
            )))
            .cache_ttl_secs(5) // 5 seconds cache (changes frequently)
            .build(),
        );

        // CPU capability from sysinfo
        caps.push(
            SysCapBuilder::new(
                "hardware:cpu".to_string(),
                "hardware".to_string(),
                "cpu".to_string(),
                "CPU".to_string(),
            )
            .version(Some(sysinfo.cpu.model.clone()))
            .amount(Some(sysinfo.cpu.cores as f64))
            .amount_dimension(Some("cores".to_string()))
            .details(Some(format!(
                "{} with {} cores @ {:.0} MHz",
                sysinfo.cpu.model, sysinfo.cpu.cores, sysinfo.cpu.frequency_mhz
            )))
            .cache_ttl_secs(600) // 10 minutes cache (changes rarely)
            .build(),
        );

        Ok(caps)
    }

    fn collect_os_caps(&self, sysinfo: &NodeSysInfo) -> Result<Vec<SysCap>, NodeInfoError> {
        let mut caps = Vec::new();

        let os = std::env::consts::OS;
        caps.push(
            SysCapBuilder::new(
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
            )
            .version(Some(sysinfo.os.version.clone()))
            .details(Some(format!(
                "Platform: {}, Version: {}, Arch: {}",
                sysinfo.os.name, sysinfo.os.version, sysinfo.os.arch
            )))
            .cache_ttl_secs(120) // 2 minutes cache (OS doesn't change much)
            .build(),
        );

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

            caps.push(
                SysCapBuilder::new(
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
                )
                .version(Some(gpu.model.clone()))
                .amount(gpu.total_memory_mb)
                .amount_dimension(if gpu.total_memory_mb.is_some() {
                    Some("MB".to_string())
                } else {
                    None
                })
                .details(Some(details))
                .cache_ttl_secs(10) // 10 seconds cache (changes frequently)
                .build(),
            );
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

            caps.push(
                SysCapBuilder::new(
                    "hardware:battery".to_string(),
                    "hardware".to_string(),
                    "battery".to_string(),
                    "Battery".to_string(),
                )
                .amount(Some(battery.percentage as f64))
                .amount_dimension(Some("percent".to_string()))
                .details(Some(format!(
                    "Status: {}, Level: {}%",
                    status, battery.percentage
                )))
                .cache_ttl_secs(3) // 3 seconds cache (battery changes frequently)
                .build(),
            );
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
        Self::new(Arc::new(SysInfoCollector::new()))
    }
}
