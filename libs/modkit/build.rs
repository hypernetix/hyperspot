use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("core_gts_schemas.zip");

    let schemas_dir = "schemas";

    // Validate presence of schemas_dir and manifest.json up front
    if !Path::new(schemas_dir).exists() {
        return Err("schemas directory not found".into());
    }
    let manifest_path = Path::new(schemas_dir).join("manifest.json");
    if !manifest_path.exists() {
        return Err("manifest.json not found in schemas directory".into());
    }

    // Create ZIP archive containing all schema files
    let file = File::create(&dest_path)?;
    let mut zip = zip::ZipWriter::new(file);

    // Add manifest.json
    let buffer = std::fs::read(&manifest_path)?;

    let options = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    zip.start_file("manifest.json", options)?;
    zip.write_all(&buffer)?;

    // Add all .schema.json files
    if Path::new(schemas_dir).exists() {
        let mut entries: Vec<_> = std::fs::read_dir(schemas_dir)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.contains(".schema."))
            {
                let buffer = std::fs::read(&path)?;

                let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                let options = zip::write::FileOptions::<()>::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o755);
                zip.start_file(file_name, options)?;
                zip.write_all(&buffer)?;
            }
        }
    }

    zip.finish()?;

    // Register individual files for rerun-if-changed to detect content changes
    // (cargo only detects directory mtime changes, not file content changes)
    println!("cargo:rerun-if-changed=Cargo.toml");

    for entry in std::fs::read_dir(schemas_dir)? {
        let entry = entry?;
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.display());
    }
    Ok(())
}
