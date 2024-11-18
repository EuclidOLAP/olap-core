fn main() {
    if let Err(e) = tonic_build::compile_protos("proto/service.proto") {
        eprintln!("Error compiling proto file: {}", e);
        std::process::exit(1);
    }
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // eprintln!(">>> OUT_DIR >>> {}", out_dir);
}
