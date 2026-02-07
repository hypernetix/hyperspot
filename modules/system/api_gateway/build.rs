#![allow(clippy::use_debug)]

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

// Using pinned version instead of @latest to ensure reproducible builds.
// The file is regenerated on each build, so using @latest would cause
// different file contents if a new version is released between builds.
// Current version: 9.0.15 (as of 2026-01-19)
const STOPLIGHT_ELEMENTS_VERSION: &str = "9.0.15";

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
            format!(
                "https://unpkg.com/@stoplight/elements@{STOPLIGHT_ELEMENTS_VERSION}/web-components.min.js"
            ),
            out_dir.join("web-components.min.js"),
        ),
        (
            format!(
                "https://unpkg.com/@stoplight/elements@{STOPLIGHT_ELEMENTS_VERSION}/styles.min.css"
            ),
            out_dir.join("styles.min.css"),
        ),
    ];

    for (url, dest) in &files {
        if let Err(e) = download_to(url.as_str(), dest) {
            println!(
                "cargo:warning=Failed to download {} -> {}: {e}",
                url,
                dest.display()
            );
            panic!(
                "Failed to download Stoplight Elements assets.\n\
                 To proceed: either build without --features embed_elements (external mode),\n\
                 or pin a specific version and vendor files manually into modules/api_gateway/assets/elements/.\n\
                 Example pinned URL: https://unpkg.com/@stoplight/elements@{STOPLIGHT_ELEMENTS_VERSION}/web-components.min.js"
            );
        }
    }
}

/// Download a file from a URL to a local path.
fn download_to(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", dest.display());
    // ureq::call() returns Err(ureq::Error::Status(...)) for 4xx/5xx responses
    let resp = ureq::get(url).call()?;
    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes)?;
    let mut f = fs::File::create(dest)?;
    f.write_all(&bytes)?;
    Ok(())
}
