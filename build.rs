fn main() {
    let euclidolap_file = "proto/euclidolap.proto";
    let olapmeta_file = "proto/olapmeta.proto";
    let agg_service_proto = "proto/agg-service.proto";
    let out_dir = "src/grpc";

    // 编译proto/euclidolap.proto
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[euclidolap_file], &["proto"])
    {
        eprintln!("Failed to compile {}, the error is: {}", euclidolap_file, e);
        std::process::exit(1);
    }

    // 编译proto/olapmeta.proto
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[olapmeta_file], &["proto"])
    {
        eprintln!("Failed to compile {}, the error is: {}", olapmeta_file, e);
        std::process::exit(1);
    }

    // 编译proto/proto/agg-service.proto
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[agg_service_proto], &["proto"])
    {
        eprintln!("Failed to compile {}, the error is: {}", agg_service_proto, e);
        std::process::exit(1);
    }

    // 文件更改监控
    println!("cargo:rerun-if-changed={}", euclidolap_file);
    println!("cargo:rerun-if-changed={}", olapmeta_file);
    println!("cargo:rerun-if-changed={}", agg_service_proto);

    // // genarate mdx parser code
    // lalrpop::process_root().unwrap_or_else(|e| {
    //     eprintln!("LALRPOP code generation failed with error: {}", e);
    //     std::process::exit(1);
    // });
    lalrpop::process_src().unwrap();

}
