use crate::model::GpuInfo;
use regex::Regex;
use std::process::Command;

#[allow(clippy::expect_used)] // good regex, it doesn't panic
static MODEL_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"Chipset Model: (.+)").expect("static regex should not panic")
});
#[allow(clippy::expect_used)] // good regex, it doesn't panic
static VRAM_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"VRAM \(.*\): (\d+) MB").expect("static regex should not panic")
});

/// Collect GPU information on macOS using `system_profiler`
pub fn collect_gpu_info() -> Vec<GpuInfo> {
    let Ok(output) = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()
    else {
        return Vec::new();
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
                model: model.as_str().trim().to_owned(),
                cores: None,
                total_memory_mb: None,
                used_memory_mb: None,
            };

            // Try to match VRAM info
            if i < vram_matches.len()
                && let Some(vram_cap) = vram_matches[i].get(1)
                && let Ok(vram_mb) = vram_cap.as_str().parse::<f64>()
            {
                gpu.total_memory_mb = Some(vram_mb);
            }

            gpus.push(gpu);
        }
    }

    gpus
}
