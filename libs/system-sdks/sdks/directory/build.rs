#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "grpc")]
    {
        println!("cargo:rerun-if-changed=proto/v1/directory.proto");
        println!("cargo:rerun-if-changed=proto");

        tonic_prost_build::configure()
            .build_client(true)
            .build_server(true)
            .compile_protos(&["proto/v1/directory.proto"], &["proto"])?;
    }

    Ok(())
}
