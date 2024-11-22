// use std::fs;
// use std::path::Path;

fn main() {
    // // 监听 proto 文件变更，强制触发重新生成
    // println!("cargo:rerun-if-changed=proto/service.proto");

    // 定义生成代码的目录
    let grpc_out_dir = "src/grpc";

    // // 检查是否是 cargo clean 操作
    // if std::env::var("CARGO_FEATURE_CARGO_CLEAN").is_ok() {
    //     if Path::new(grpc_out_dir).exists() {
    //         if let Err(e) = fs::remove_dir_all(grpc_out_dir) {
    //             eprintln!("Failed to clean grpc directory {}: {}", grpc_out_dir, e);
    //         } else {
    //             println!("cargo:warning=Cleaned gRPC directory: {}", grpc_out_dir);
    //         }
    //     }
    // }

    // // 创建 gRPC 输出目录
    // if !Path::new(grpc_out_dir).exists() {
    //     if let Err(e) = fs::create_dir_all(grpc_out_dir) {
    //         eprintln!("Failed to create gRPC directory {}: {}", grpc_out_dir, e);
    //         std::process::exit(1);
    //     }
    // }

    // 使用 tonic-build 编译 .proto 文件
    if let Err(e) = tonic_build::configure()
        .out_dir(grpc_out_dir)
        .compile(&["proto/service.proto"], &["proto"])
    {
        eprintln!("Failed to compile proto files: {}", e);
        std::process::exit(1);
    }
}
