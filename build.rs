fn main() -> std::io::Result<()> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    prost_build::compile_protos(&["protos/detections.proto"], &["protos/"])?;
    Ok(())
}