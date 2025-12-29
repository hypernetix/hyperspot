//! This example downloads the latest version of Stoplight Elements assets and
//! embeds them into the API Ingress module.
//!
//! Usage:
//!
//! ```sh
//! cargo run --example update_assets
//! ```

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = base_path.join("assets").join("elements");
    if let Err(e) = fs::create_dir_all(&out_dir) {
        panic!("Failed to create assets directory for embedded Elements.\n{e:?}");
    }
    let files = [
        (
            "https://unpkg.com/@stoplight/elements@latest/web-components.min.js",
            out_dir.join("web-components.min.js"),
        ),
        (
            "https://unpkg.com/@stoplight/elements@latest/styles.min.css",
            out_dir.join("styles.min.css"),
        ),
    ];

    for (url, dest) in &files {
        if let Err(e) = download_to(url, dest).await {
            panic!("Failed to download Stoplight Elements assets.\n{e:?}");
        }
    }
}

async fn download_to(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {}", resp.status(), url).into());
    }
    let bytes = resp.bytes().await?;
    fs::File::create(dest)?.write_all(&bytes)?;
    Ok(())
}
