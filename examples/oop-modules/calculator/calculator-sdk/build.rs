fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/oop/calculator/v1/accum.proto");
    println!("cargo:rerun-if-changed=proto");

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/oop/calculator/v1/accum.proto"], &["proto"])?;

    Ok(())
}
