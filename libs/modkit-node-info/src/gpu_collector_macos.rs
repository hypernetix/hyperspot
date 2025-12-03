use crate::model::GpuInfo;
use once_cell::sync::Lazy;
use regex::Regex;
use std::process::Command;

static MODEL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"Chipset Model: (.+)").unwrap());
static VRAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"VRAM \(.*\): (\d+) MB").unwrap());

/// Collect GPU information on macOS using system_profiler
pub fn collect_gpu_info() -> Vec<GpuInfo> {
    let output = match Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()
    {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    // Parse GPU model names
    let model_matches: Vec<_> = MODEL_REGEX.captures_iter(&output_str).collect();
    let vram_matches: Vec<_> = VRAM_REGEX.captures_iter(&output_str).collect();

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

    gpus
}
