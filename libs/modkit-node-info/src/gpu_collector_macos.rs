use crate::error::NodeInfoError;
use crate::model::GpuInfo;
use regex::Regex;
use std::process::Command;

/// Collect GPU information on macOS using system_profiler
pub fn collect_gpu_info() -> Result<Vec<GpuInfo>, NodeInfoError> {
    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()
        .map_err(|e| {
            NodeInfoError::SysInfoCollectionFailed(format!("Failed to run system_profiler: {}", e))
        })?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    // Parse GPU model names
    let model_regex = Regex::new(r"Chipset Model: (.+)")
        .map_err(|e| NodeInfoError::Internal(format!("Regex error: {}", e)))?;
    let vram_regex = Regex::new(r"VRAM \(.*\): (\d+) MB")
        .map_err(|e| NodeInfoError::Internal(format!("Regex error: {}", e)))?;

    let model_matches: Vec<_> = model_regex.captures_iter(&output_str).collect();
    let vram_matches: Vec<_> = vram_regex.captures_iter(&output_str).collect();

    for (i, model_cap) in model_matches.iter().enumerate() {
        if let Some(model) = model_cap.get(1) {
            let mut gpu = GpuInfo {
                model: model.as_str().trim().to_string(),
                cores: None,
                total_memory_mb: None,
                used_memory_mb: None,
            };

            // Try to match VRAM info
            if i < vram_matches.len() {
                if let Some(vram) = vram_matches[i].get(1) {
                    if let Ok(vram_mb) = vram.as_str().parse::<f64>() {
                        gpu.total_memory_mb = Some(vram_mb);
                    }
                }
            }

            gpus.push(gpu);
        }
    }

    Ok(gpus)
}
