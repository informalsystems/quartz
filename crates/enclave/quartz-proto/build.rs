fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/prost")
        .compile(&["proto/quartz.proto"], &["proto"])?;
    Ok(())
}
