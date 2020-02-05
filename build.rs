fn main() -> Result<(), std::io::Error> {
    let mut config = prost_build::Config::default();
    config.out_dir("src/protobuf");

    config.compile_protos(&["src/protobuf/gtfs_realtime.proto"], &["src/protobuf"])?;
    Ok(())
}
