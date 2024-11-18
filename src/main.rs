// use std::{thread, time};
use grpc_service::greeter_server::{Greeter, GreeterServer};
use grpc_service::{ClientRequest, ServerResponse};
use tonic::{transport::Server, Request, Response, Status};

// 自动生成的 gRPC 模块
pub mod grpc_service {
    tonic::include_proto!("grpc_service"); // 包名要与 .proto 文件中的 package 名字匹配
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<ClientRequest>, // 从客户端接收的请求
    ) -> Result<Response<ServerResponse>, Status> {
        println!("Received request: {:?}", request);

        // 获取请求中的数据
        let client_message = request.into_inner().message;

        // 创建响应
        let reply = grpc_service::ServerResponse {
            reply: format!("Hello, {}!", client_message),
        };

        // 返回响应
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 定义服务端监听地址
    let addr = "127.0.0.1:50052".parse().unwrap();

    // 创建 Greeter 服务实例
    let greeter = MyGreeter::default();

    println!(">>>>>> EuclidOLAP Server is listening on {} >>>>>>", addr);

    // 启动 gRPC 服务端
    Server::builder()
        .add_service(GreeterServer::new(greeter)) // 添加 Greeter 服务
        .serve(addr) // 启动服务端
        .await?;

    Ok(())
}
