use crate::model::GpuInfo;
use std::process::Command;

/// Collect GPU information on Linux using NVML for NVIDIA GPUs, fallback to lspci
pub fn collect_gpu_info() -> Vec<GpuInfo> {
    // Try NVML first for NVIDIA GPUs
    if let Some(nvidia_gpus) = collect_nvidia_gpus() {
        if !nvidia_gpus.is_empty() {
            tracing::debug!("Found {} NVIDIA GPU(s) via NVML", nvidia_gpus.len());
            return nvidia_gpus;
        }
    }

    // Fallback to lspci for AMD/Intel/other GPUs
    tracing::debug!("NVML not available, falling back to lspci");
    collect_gpus_via_lspci()
}

/// Collect NVIDIA GPU information using NVML
fn collect_nvidia_gpus() -> Option<Vec<GpuInfo>> {
    use nvml_wrapper::Nvml;

    // Initialize NVML
    let nvml = match Nvml::init() {
        Ok(nvml) => nvml,
        Err(_) => return None,
    };

    let device_count = match nvml.device_count() {
        Ok(count) => count,
        Err(_) => return None,
    };

    let mut gpus = Vec::new();

    for i in 0..device_count {
        match nvml.device_by_index(i) {
            Ok(device) => {
                let model = device
                    .name()
                    .unwrap_or_else(|_| "Unknown NVIDIA GPU".to_string());

                // Get memory info
                let memory_info = device.memory_info().ok();
                let total_memory_mb = memory_info
                    .as_ref()
                    .map(|m| m.total as f64 / 1024.0 / 1024.0);
                let used_memory_mb = memory_info
                    .as_ref()
                    .map(|m| m.used as f64 / 1024.0 / 1024.0);

                // Try to get CUDA cores (not directly available, but we can get compute capability)
                let cores = None; // NVML doesn't expose CUDA cores directly

                gpus.push(GpuInfo {
                    model,
                    cores,
                    total_memory_mb,
                    used_memory_mb,
                });

                tracing::debug!(
                    "NVIDIA GPU {}: {} (Memory: {:.0} MB / {:.0} MB)",
                    i,
                    gpus.last().unwrap().model,
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

/// Collect GPU information using lspci (fallback for non-NVIDIA GPUs)
fn collect_gpus_via_lspci() -> Vec<GpuInfo> {
    let output = Command::new("lspci").output();

    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut gpus = Vec::new();

            for line in output_str.lines() {
                let line_lower = line.to_lowercase();
                if line_lower.contains("vga")
                    || line_lower.contains("3d")
                    || line_lower.contains("display")
                {
                    // Extract GPU model from lspci output
                    // Format: "00:02.0 VGA compatible controller: Intel Corporation ..."
                    if let Some(pos) = line.find(':') {
                        if let Some(model_start) = line[pos..].find(':') {
                            let model = line[pos + model_start + 1..].trim().to_string();
                            gpus.push(GpuInfo {
                                model,
                                cores: None,
                                total_memory_mb: None,
                                used_memory_mb: None,
                            });
                        }
                    }
                }
            }

            if !gpus.is_empty() {
                tracing::debug!("Found {} GPU(s) via lspci", gpus.len());
                return gpus;
            }
        }
    }

    // If lspci fails or finds nothing, return empty list
    Vec::new()
}
