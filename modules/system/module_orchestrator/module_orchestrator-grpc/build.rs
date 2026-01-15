use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    // From module_orchestrator-grpc, go up 4 levels to workspace root:
    // grpc -> module_orchestrator -> system -> modules -> root
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or("Could not find workspace root")?;

    let proto_dir = workspace_root.join("proto");
    let proto_file = proto_dir.join("directory/v1/directory.proto");

    // Verify paths exist
    if !proto_file.exists() {
        return Err(format!(
            "Proto file not found: {} (workspace root: {})",
            proto_file.display(),
            workspace_root.display()
        )
        .into());
    }

    println!("cargo:rerun-if-changed={}", proto_file.display());
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    // Configure tonic_prost_build
    // Note: We don't set extern_path for .google.protobuf as it may already be set
    // by tonic_prost_build internally. If protoc can't find the includes,
    // prost-build will use bundled types automatically.
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(
            &[proto_file.to_str().ok_or("Invalid proto file path")?],
            &[proto_dir.to_str().ok_or("Invalid proto dir path")?],
        )?;

    Ok(())
}
