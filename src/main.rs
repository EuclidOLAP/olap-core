use euclidolap::olap_api_server::{OlapApi, OlapApiServer};
use euclidolap::{OlapRequest, OlapResponse, Row};
use tonic::{transport::Server, Request, Response, Status};

mod olapmeta_grpc_client;
mod mdd;
mod mdx_statements;

mod euclidolap {
    include!("grpc/euclidolap.rs");
}

pub mod mdx_ast;
pub mod mdx_lexer;
pub mod mdx_tokens;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub mdx_grammar);

use crate::mdx_grammar::SelectionMDXParser;

use crate::mdx_lexer::Lexer as MdxLexer;

#[derive(Debug, Default)]
pub struct EuclidOLAPService {}

#[tonic::async_trait]
impl OlapApi for EuclidOLAPService {
    async fn execute_operation(
        &self,
        request: Request<OlapRequest>, // 从客户端接收的请求
    ) -> Result<Response<OlapResponse>, Status> {
        // println!("Received request: {:?}", request);

        // 从请求中解析操作类型和语句
        let olap_request = request.into_inner();
        let operation_type = olap_request.operation_type;
        let statement = olap_request.statement;

        // println!(
        //     "Operation Type: {}, Statement: >>>>>>{}<<<<<<",
        //     operation_type, statement
        // );

        handle_stat(operation_type, statement).await;

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

async fn handle_stat(optype: String, statement: String) -> () {
    match optype.as_str() {
        "MDX" => {
            // println!("\n\n\n>>> @@@ Operation Type: {}", optype);
            // println!(">>> @@@ Statement: {}", statement);
            let ast_selstat = SelectionMDXParser::new()
                .parse(MdxLexer::new(statement.as_str()))
                .unwrap();

            exe_md_query(ast_selstat).await;
        }
        _ => {
            panic!(
                "In fn `handle_stat()`: Unexpected operation type: {}",
                optype
            );
        }
    }
    ()
}

async fn exe_md_query(ast_selstat: mdx_ast::AstSelectionStatement) -> () {
    // println!("---+++ exe_md_query +++--- {:#?}", ast_selstat);

    // 生成多维查询上下文
    // let context = tokio::runtime::Runtime::new().unwrap().block_on(ast_selstat.gen_md_context());
    let mut context = ast_selstat.gen_md_context().await;
    // println!("Multi-Dimensonal Query Context:\n{:#?}", context);

    /*
     * 构建真实的多维查询坐标轴
     */
    let _axes = ast_selstat.build_axes(&mut context).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mdx_statements::*;

    #[test]
    fn test_handle_stat() {
        tokio::runtime::Runtime::new().unwrap().block_on(
            handle_stat(String::from("MDX"), _mdx_demo())    
        );
    }

    #[test]
    fn test_handle_stat_2() {
        tokio::runtime::Runtime::new().unwrap().block_on(
            handle_stat(String::from("MDX"), _mdx_demo_2_axposstr())    
        );
    }

    #[test]
    fn test_grpc_locate_gid_1() {

        async fn test_grpc_locate_gid_1_async() {

            use mdd::MultiDimensionalEntity;
            use olapmeta_grpc_client::GrpcClient;

            let params: [(u64, u64); 9] = [
                (600000000000023	, 300000000004617),
                (600000000000022	, 300000000004173),
                (600000000000015	, 300000000004175),
                (600000000000015	, 300000000004072),
                (600000000000015	, 300000000004164),
                (600000000000008	, 300000000004531),
                (600000000000011	, 300000000000003),
                (600000000000023	, 300000000004612),
                (600000000000024	, 300000000004612),
            ];

            let mut grpc_cli = GrpcClient::new("http://192.168.66.51:50051".to_string())
                .await
                .expect("Failed to create client");

            for (origin_gid, target_entity_gid) in params {
                let olap_entity
                    = grpc_cli.locate_universal_olap_entity_by_gid(origin_gid, target_entity_gid)
                    .await.unwrap();
    
                match olap_entity {
                    MultiDimensionalEntity::MemberWrap(member) => {
                        println!(">>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>>--->>> Member: \n{:#?}", member);
                    }
                    _ => {
                        panic!("Unexpected olap_entity type: {:#?}", olap_entity);
                    }
                }
            }

        }

        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(test_grpc_locate_gid_1_async());

    }
}