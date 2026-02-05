use std::io::Read;
use tracing::info;
use zip;

/// Get core GTS schemas provided by the modkit framework.
///
/// These schemas are loaded from an embedded ZIP archive containing JSON schema files.
/// The schemas are fundamental types that other modules build upon.
/// Examples: `BaseModkitPluginV1` for plugin systems.
///
/// # Errors
///
/// Returns an error if the embedded ZIP archive cannot be read or if any schema
/// file cannot be parsed as valid JSON.
pub fn get_core_gts_schemas() -> anyhow::Result<Vec<serde_json::Value>> {
    info!("Loading core GTS schemas from embedded archive");

    let mut schemas = Vec::new();

    // Load embedded ZIP archive created by build script
    let zip_data = include_bytes!(concat!(env!("OUT_DIR"), "/core_gts_schemas.zip"));
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Extract and parse each schema file
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_owned();

        // Skip manifest.json, only process .schema.json files
        if file_name.ends_with(".schema.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let schema_json: serde_json::Value = serde_json::from_str(&contents)
                .map_err(|e| anyhow::anyhow!("Failed to parse schema {file_name}: {e}"))?;

            schemas.push(schema_json);
            info!("Loaded core GTS schema: {}", file_name);
        }
    }

    if schemas.is_empty() {
        return Err(anyhow::anyhow!(
            "no core GTS schemas found in embedded archive"
        ));
    }

    info!("Core GTS schemas loaded: {} schemas", schemas.len());
    Ok(schemas)
}
