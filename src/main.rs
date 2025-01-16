use euclidolap::olap_api_server::{OlapApi, OlapApiServer};
use euclidolap::{OlapRequest, OlapResponse, Row};
use tonic::{transport::Server, Request, Response, Status};

// mod mdx_parser;

mod euclidolap {
    include!("grpc/euclidolap.rs");
}

#[derive(Debug, Default)]
pub struct EuclidOLAPService {}

#[tonic::async_trait]
impl OlapApi for EuclidOLAPService {
    async fn execute_operation(
        &self,
        request: Request<OlapRequest>, // 从客户端接收的请求
    ) -> Result<Response<OlapResponse>, Status> {
        println!("Received request: {:?}", request);

        // 从请求中解析操作类型和语句
        let olap_request = request.into_inner();
        let operation_type = olap_request.operation_type;
        let statement = olap_request.statement;

        println!(
            "Operation Type: {}, Statement: {}",
            operation_type, statement
        );

        // 伪造一个响应，返回结果
        let response = OlapResponse {
            rows: vec![
                Row {
                    columns: vec![
                        "[ cell: R 0 C 0 ]".to_string(),
                        "[ cell: R 0 C 1 ]".to_string(),
                        "[ cell: R 0 C 2 ]".to_string(),
                    ],
                },
                Row {
                    columns: vec![
                        "[ cell: R 1 C 0 ]".to_string(),
                        "[ cell: R 1 C 1 ]".to_string(),
                        "[ cell: R 1 C 2 ]".to_string(),
                    ],
                },
                Row {
                    columns: vec![
                        "AAAsssDDD".to_string(),
                        "111222333".to_string(),
                        "@@@###$$$".to_string(),
                    ],
                },
            ],
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 定义服务端监听地址
    // let addr = "127.0.0.1:50052".parse().unwrap();
    let addr = "0.0.0.0:50052".parse().unwrap();

    // 创建 Greeter 服务实例
    let olap_api_server = EuclidOLAPService::default();

    println!(">>> EuclidOLAP Server is listening on {} <<<", addr);

    // 启动 gRPC 服务端
    Server::builder()
        .add_service(OlapApiServer::new(olap_api_server)) // 添加 Greeter 服务
        .serve(addr) // 启动服务端
        .await?;

    Ok(())
}
