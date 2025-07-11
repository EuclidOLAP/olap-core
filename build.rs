fn main() {
    let proto_files = ["proto/euclidolap.proto", "proto/olapmeta.proto", "proto/agg-service.proto"];

    let proto_includes = ["proto"];

    // OUT_DIR is a special env var provided by Cargo (like target/debug/build/.../out)
    let out_dir = std::env::var("OUT_DIR").unwrap();

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir(&out_dir) // Output to OUT_DIR instead of src/
        .compile(&proto_files, &proto_includes)
        .expect("Failed to compile proto files");

    // Rerun build script if proto files change
    for proto in proto_files.iter() {
        println!("cargo:rerun-if-changed={}", proto);
    }

    lalrpop::process_src().unwrap();
}
