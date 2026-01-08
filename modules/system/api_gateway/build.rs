#![allow(clippy::use_debug)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    // Only run when the embed_elements feature is enabled
    let embed_enabled = env::var("CARGO_FEATURE_EMBED_ELEMENTS").is_ok();
    if !embed_enabled {
        return;
    }

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBED_ELEMENTS");

    let out_dir = Path::new("assets").join("elements");
    if let Err(e) = fs::create_dir_all(&out_dir) {
        println!("cargo:warning=Failed to create assets/elements directory: {e}");
        panic!(
            "Failed to create assets directory for embedded Elements. Build with --no-default-features or without --features embed_elements, or vendor assets manually."
        );
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
        if let Err(e) = download_to(url, dest) {
            println!(
                "cargo:warning=Failed to download {url} -> {}: {e}",
                dest.display()
            );
            panic!(
                "Failed to download Stoplight Elements assets.\n\
                 To proceed: either build without --features embed_elements (external mode),\n\
                 or pin a specific version and vendor files manually into modules/api_gateway/assets/elements/.\n\
                 Example pinned URL: https://unpkg.com/@stoplight/elements@7.18.0/web-components.min.js"
            );
        }
    }
}

fn download_to(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", dest.display());
    let resp = reqwest::blocking::get(url)?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {}", resp.status(), url).into());
    }
    let bytes = resp.bytes()?;
    let mut f = fs::File::create(dest)?;
    f.write_all(&bytes)?;
    Ok(())
}
