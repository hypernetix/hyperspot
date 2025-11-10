fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/directory/v1/directory.proto");
    println!("cargo:rerun-if-changed=proto");

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/directory/v1/directory.proto"], &["proto"])?;
    Ok(())
}
