fn main() {
    let service_file = "proto/service.proto";
    let euclidolap_file = "proto/euclidolap.proto";
    let out_dir = "src/grpc";

    // 编译proto/service.proto
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[service_file], &["proto"])
    {
        eprintln!("Failed to compile {}, the error is: {}", service_file, e);
        std::process::exit(1);
    }

    // 编译proto/euclidolap.proto
    if let Err(e) = tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[euclidolap_file], &["proto"])
    {
        eprintln!("Failed to compile {}, the error is: {}", euclidolap_file, e);
        std::process::exit(1);
    }

    // 文件更改监控
    println!("cargo:rerun-if-changed={}", service_file);
    println!("cargo:rerun-if-changed={}", euclidolap_file);

    // // genarate mdx parser code
    // lalrpop::process_root().unwrap_or_else(|e| {
    //     eprintln!("LALRPOP code generation failed with error: {}", e);
    //     std::process::exit(1);
    // });
}
