fn main() {
    // Paths to .proto files used for gRPC service definitions
    let euclidolap_file = "proto/euclidolap.proto";
    let olapmeta_file = "proto/olapmeta.proto";
    let agg_service_proto = "proto/agg-service.proto";

    // Output directory for generated Rust code
    let out_dir = "src/grpc";

    // Compile euclidolap.proto to Rust using tonic
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[euclidolap_file], &["proto"])
    {
        eprintln!("Failed to compile {}, the error is: {}", euclidolap_file, e);
        std::process::exit(1);
    }

    // Compile olapmeta.proto to Rust using tonic
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[olapmeta_file], &["proto"])
    {
        eprintln!("Failed to compile {}, the error is: {}", olapmeta_file, e);
        std::process::exit(1);
    }

    // Compile agg-service.proto to Rust using tonic
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[agg_service_proto], &["proto"])
    {
        eprintln!(
            "Failed to compile {}, the error is: {}",
            agg_service_proto, e
        );
        std::process::exit(1);
    }

    // Instruct Cargo to rerun the build script if any of the .proto files change
    println!("cargo:rerun-if-changed={}", euclidolap_file);
    println!("cargo:rerun-if-changed={}", olapmeta_file);
    println!("cargo:rerun-if-changed={}", agg_service_proto);

    // Run LALRPOP to process grammar files into Rust code
    lalrpop::process_src().unwrap();
}
