use crate::model::GpuInfo;
use std::process::Command;

/// Collect GPU information on Windows using NVML for NVIDIA GPUs, fallback to WMIC
pub fn collect_gpu_info() -> Vec<GpuInfo> {
    // Try NVML first for NVIDIA GPUs
    if let Some(nvidia_gpus) = collect_nvidia_gpus() {
        if !nvidia_gpus.is_empty() {
            tracing::debug!("Found {} NVIDIA GPU(s) via NVML", nvidia_gpus.len());
            return nvidia_gpus;
        }
    }

    // Fallback to WMIC for other GPUs
    tracing::debug!("NVML not available, falling back to WMIC");
    collect_gpus_via_wmic()
}

/// Collect NVIDIA GPU information using NVML
#[allow(clippy::cast_precision_loss)]
fn collect_nvidia_gpus() -> Option<Vec<GpuInfo>> {
    use nvml_wrapper::Nvml;

    // Initialize NVML
    let Ok(nvml) = Nvml::init() else {
        return None;
    };

    let Ok(device_count) = nvml.device_count() else {
        return None;
    };

    let mut gpus = Vec::new();

    for i in 0..device_count {
        match nvml.device_by_index(i) {
            Ok(device) => {
                let model = device
                    .name()
                    .unwrap_or_else(|_| "Unknown NVIDIA GPU".to_owned());

                // Get memory info
                let memory_info = device.memory_info().ok();
                let total_memory_mb = memory_info
                    .as_ref()
                    .map(|m| m.total as f64 / 1024.0 / 1024.0);
                let used_memory_mb = memory_info
                    .as_ref()
                    .map(|m| m.used as f64 / 1024.0 / 1024.0);

                // NVML doesn't expose CUDA cores directly
                let cores = None;

                gpus.push(GpuInfo {
                    model,
                    cores,
                    total_memory_mb,
                    used_memory_mb,
                });

                tracing::debug!(
                    "NVIDIA GPU {}: {} (Memory: {:.0} MB / {:.0} MB)",
                    i,
                    gpus.last()?.model,
                    used_memory_mb.unwrap_or(0.0),
                    total_memory_mb.unwrap_or(0.0)
                );
            }
            Err(e) => {
                tracing::warn!("Failed to get NVML device handle for GPU #{}: {}", i, e);
            }
        }
    }

    Some(gpus)
}

/// Collect GPU information using WMIC (fallback for non-NVIDIA GPUs)
#[allow(clippy::cast_precision_loss)]
fn collect_gpus_via_wmic() -> Vec<GpuInfo> {
    let output = Command::new("wmic")
        .args([
            "path",
            "win32_VideoController",
            "get",
            "name,AdapterRAM",
            "/format:csv",
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut gpus = Vec::new();

            // Skip header line and parse CSV output
            for line in output_str.lines().skip(1) {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    let name = parts[1].trim();
                    if !name.is_empty() {
                        let mut gpu = GpuInfo {
                            model: name.to_owned(),
                            cores: None,
                            total_memory_mb: None,
                            used_memory_mb: None,
                        };

                        // Parse memory if available
                        if let Ok(ram_bytes) = parts[2].trim().parse::<u64>() {
                            if ram_bytes > 0 {
                                gpu.total_memory_mb = Some(ram_bytes as f64 / (1024.0 * 1024.0));
                            }
                        }

                        gpus.push(gpu);
                    }
                }
            }

            if !gpus.is_empty() {
                tracing::debug!("Found {} GPU(s) via WMIC", gpus.len());
            }
            return gpus;
        }
    }

    Vec::new()
}
