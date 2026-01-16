use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("core_gts_schemas.zip");

    // Create ZIP archive containing all schema files
    let file = File::create(&dest_path)?;
    let mut zip = zip::ZipWriter::new(file);

    let schemas_dir = "schemas";

    // Add manifest.json
    let manifest_path = Path::new(schemas_dir).join("manifest.json");
    if manifest_path.exists() {
        let buffer = std::fs::read(&manifest_path)?;

        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        zip.start_file("manifest.json", options)?;
        zip.write_all(&buffer)?;
    }

    // Add all .schema.json files
    if Path::new(schemas_dir).exists() {
        for entry in std::fs::read_dir(schemas_dir)? {
            let entry = entry?;
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

    println!("cargo:rerun-if-changed=schemas/");
    Ok(())
}
